// JeebsAI Evolution Dashboard — fully self-contained
const ROOT_ADMIN = typeof JEEBS_ROOT_ADMIN !== "undefined" ? JEEBS_ROOT_ADMIN : null;

let allUpdates = [];
let userRole = "guest";
let rootAdmin = false;
let brainStats = null;
let currentPage = 1;
const PAGE_SIZE = 8;

// ── Utilities ──────────────────────────────────────────────
function el(id) { return document.getElementById(id); }
function setText(id, txt) { const e = el(id); if (e) e.textContent = txt; }
function setHtml(id, html) { const e = el(id); if (e) e.innerHTML = html; }

function escapeHtml(t) {
    if (!t) return "";
    return String(t).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;");
}

function fmtDate(raw) {
    if (!raw) return "n/a";
    const d = new Date(raw);
    return isNaN(d.getTime()) ? raw : d.toLocaleString();
}

function fmtMs(ms) {
    const v = Number(ms || 0);
    return v < 1000 ? v + "ms" : (v / 1000).toFixed(1) + "s";
}

function setStatus(msg, err) {
    const s = el("status-message");
    if (!s) return;
    s.textContent = msg || "";
    s.style.color = err ? "#ef9a9a" : "#90caf9";
}

async function parseErr(res, fb) {
    try { const j = await res.json(); return j.error || j.message || fb; } catch {}
    try { return await res.text() || fb; } catch {}
    return fb;
}

// ── Auth Banner ────────────────────────────────────────────
function showAuthBanner(auth) {
    const b = el("auth-debug-banner");
    if (!b) return;
    if (!auth || !auth.loggedIn) {
        b.innerHTML = '<div style="font-size:0.9em; color:var(--muted)">Not signed in &mdash; <a href="/webui/index.html" style="color:var(--accent)">Sign in</a> for admin controls</div>';
    } else {
        b.innerHTML = '<div style="font-size:0.9em; color:var(--muted)">Signed in as: <strong>' + escapeHtml(auth.username || "(none)") + '</strong> &mdash; admin: <strong>' + (auth.isAdmin ? "yes" : "no") + '</strong></div>';
    }
}

// ── Brain Stats (public, no auth needed) ───────────────────
async function loadBrainStats() {
    try {
        const res = await safeFetch("/api/evolution/stats");
        if (!res.ok) { console.warn("Stats endpoint returned", res.status); return; }
        brainStats = await res.json();
        renderBrainStats();
    } catch (e) {
        console.error("loadBrainStats error:", e);
    }
}

function renderBrainStats() {
    if (!brainStats) return;
    const b = brainStats.brain || {};
    const t = brainStats.thinker || {};
    const p = brainStats.proposals || {};

    setText("m-brain-nodes", (b.nodes ?? 0).toLocaleString());
    setText("m-learned-facts", (b.learned_facts ?? 0).toLocaleString());
    setText("m-chats-24h", (b.chat_logs_24h ?? 0).toLocaleString());
    setText("m-proposals", (p.total ?? 0).toLocaleString());
    setText("m-cycles", (t.total_cycles ?? 0).toLocaleString());
    setText("m-unanswered", (b.unanswered_24h ?? 0).toLocaleString());
    setText("m-warnings", (b.warnings_24h ?? 0).toLocaleString());
    setText("m-errors", (b.errors_24h ?? 0).toLocaleString());

    // Thinker panel
    const dot = el("thinker-dot");
    const status = t.status || "unknown";
    setText("thinker-status", status.toUpperCase());
    if (dot) {
        dot.className = status === "running" ? "live-dot" : "live-dot off";
    }

    const drive = b.knowledge_drive ?? 0;
    setText("thinker-drive", (drive * 100).toFixed(0) + "%");
    const bar = document.querySelector("#drive-bar .progress-fill");
    if (bar) bar.style.width = (drive * 100).toFixed(0) + "%";

    setText("thinker-last-cycle", fmtDate(t.last_cycle_at));
    setText("thinker-reason", t.last_reason || "n/a");

    // Unknown topics
    const topics = b.top_unknown_topics || [];
    if (topics.length === 0) {
        setHtml("unknown-topics", '<div style="padding:8px">No unknown topics detected. The brain is handling all questions.</div>');
    } else {
        setHtml("unknown-topics", topics.map(function(tp) {
            return '<span style="display:inline-block; margin:3px; padding:4px 10px; background:var(--panel-2); border:1px solid var(--border); border-radius:6px; font-size:0.85rem">' + escapeHtml(tp) + '</span>';
        }).join(""));
    }
}

// ── Evolution Updates ──────────────────────────────────────
async function loadUpdates() {
    try {
        const res = await safeFetch("/api/evolution/updates", {
            credentials: "same-origin",
            headers: authHeaders(),
        });
        if (!res.ok) {
            setStatus(await parseErr(res, "Failed to load proposals."), true);
            setHtml("updates-list", '<div class="card"><div class="muted" style="padding:20px; text-align:center">Unable to load proposals.</div></div>');
            return;
        }
        const data = await res.json();
        allUpdates = data.updates || [];
        // Use API role if it's admin (confirms session works)
        if (data.role === "admin") {
            userRole = "admin";
            rootAdmin = true;
        }
        setStatus("Loaded " + allUpdates.length + " evolution proposal(s).");
        currentPage = 1;
        renderUpdates();
    } catch (e) {
        console.error("loadUpdates error:", e);
        setStatus("Error loading proposals: " + e.message, true);
    }
}

function renderUpdates() {
    const filter = el("statusFilter") ? el("statusFilter").value : "all";
    const filtered = filter === "all" ? allUpdates : allUpdates.filter(function(u) { return u.status === filter; });
    const total = filtered.length;
    const pages = Math.max(1, Math.ceil(total / PAGE_SIZE));
    if (currentPage > pages) currentPage = pages;
    const start = (currentPage - 1) * PAGE_SIZE;
    const pageItems = filtered.slice(start, start + PAGE_SIZE);

    const container = el("updates-list");
    if (!container) return;
    container.innerHTML = "";

    if (total === 0) {
        container.innerHTML = '<div class="card" style="text-align:center; padding:30px"><div style="font-size:2em; margin-bottom:8px">&#x1F4AD;</div><div class="muted">No evolution proposals yet. Click <strong>Think Now</strong> to trigger a thinking cycle.</div></div>';
        return;
    }

    // Summary stats
    var stats = { pending:0, applied:0, denied:0 };
    filtered.forEach(function(u) { if (stats[u.status] !== undefined) stats[u.status]++; });
    var pp = brainStats ? brainStats.proposals : null;

    var summaryHtml = '<div class="grid cols-3" style="margin-bottom:16px">';
    summaryHtml += '<div class="card"><h3>Proposals</h3><div class="stack"><div><strong>Showing:</strong> ' + total + '</div><div><strong>Pending:</strong> ' + stats.pending + '</div><div><strong>Applied:</strong> ' + stats.applied + '</div></div></div>';
    summaryHtml += '<div class="card"><h3>Severity</h3><div class="stack">';
    var sevCounts = { high:0, medium:0, low:0 };
    filtered.forEach(function(u) { var s = (u.severity||"low").toLowerCase(); if (sevCounts[s] !== undefined) sevCounts[s]++; });
    summaryHtml += '<div><strong>High:</strong> ' + sevCounts.high + '</div>';
    summaryHtml += '<div><strong>Medium:</strong> ' + sevCounts.medium + '</div>';
    summaryHtml += '<div><strong>Low:</strong> ' + sevCounts.low + '</div>';
    summaryHtml += '</div></div>';
    summaryHtml += '<div class="card"><h3>Activity</h3><div class="stack">';
    summaryHtml += '<div><strong>Brain nodes:</strong> ' + ((brainStats && brainStats.brain) ? brainStats.brain.nodes : "-") + '</div>';
    summaryHtml += '<div><strong>Think cycles:</strong> ' + ((brainStats && brainStats.thinker) ? brainStats.thinker.total_cycles : "-") + '</div>';
    summaryHtml += '<div><strong>Knowledge drive:</strong> ' + ((brainStats && brainStats.brain) ? ((brainStats.brain.knowledge_drive * 100).toFixed(0) + "%") : "-") + '</div>';
    summaryHtml += '</div></div></div>';
    container.insertAdjacentHTML("beforeend", summaryHtml);

    // Proposal cards
    var grid = document.createElement("div");
    grid.className = "grid cols-2";
    grid.style.gridTemplateColumns = "repeat(auto-fit, minmax(320px, 1fr))";

    pageItems.forEach(function(u) {
        var card = document.createElement("div");
        card.className = "card";
        var sevClass = (u.severity || "low").toLowerCase();

        var changesHtml = (u.changes || []).map(function(c) {
            return '<div style="margin-bottom:8px"><strong>' + escapeHtml(c.path) + '</strong><pre style="white-space:pre-wrap;max-height:160px;overflow:auto;background:transparent;border:1px solid var(--border);padding:8px;border-radius:6px;font-size:0.8em">' + escapeHtml(c.new_content || "") + '</pre></div>';
        }).join("");

        var commentsHtml = (u.comments || []).map(function(c) {
            return '<div style="background:#0e1114;padding:8px;margin:4px 0;font-size:0.9em;border-radius:6px"><strong>' + escapeHtml(c.author) + '</strong>: ' + escapeHtml(c.content) + '</div>';
        }).join("") || '<div class="muted" style="font-style:italic;font-size:0.85em">No comments yet.</div>';

        var actions = '<div style="display:flex;gap:8px;align-items:center"><button class="btn" onclick="voteUpdate(\'' + u.id + '\',\'up\')">&#x1F44D; ' + (u.votes_up || 0) + '</button><button class="btn" onclick="voteUpdate(\'' + u.id + '\',\'down\')">&#x1F44E; ' + (u.votes_down || 0) + '</button></div>';
        if (rootAdmin && u.status === "pending") {
            actions += '<div style="margin-top:6px;display:flex;gap:6px;flex-wrap:wrap"><button class="btn accent" onclick="applyUpdate(\'' + u.id + '\')">Apply</button><button class="btn danger" onclick="denyUpdate(\'' + u.id + '\')">Deny</button><button class="btn" onclick="resolveUpdate(\'' + u.id + '\')">Resolve</button></div>';
        } else if (rootAdmin && u.status === "applied") {
            actions += '<div style="margin-top:6px"><button class="btn danger" onclick="rollbackUpdate(\'' + u.id + '\')">Rollback</button></div>';
        }

        var feeling = (u.feeling || "").toLowerCase();
        var feelingIcon = feeling === "like" ? "&#x1F44D;" : feeling === "dislike" ? "&#x1F44E;" : (feeling ? "&#x1F610;" : "");

        card.innerHTML = '<div class="update-header"><h3 style="margin:0;flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis">' + escapeHtml(u.title || "(untitled)") + '</h3><div style="flex-shrink:0"><span class="severity ' + sevClass + '">' + escapeHtml(u.severity || "Low") + '</span><span class="status ' + u.status + '" style="margin-left:4px">' + escapeHtml(u.status || "unknown") + '</span>' + (feelingIcon ? '<span style="margin-left:4px">' + feelingIcon + '</span>' : '') + '</div></div>' +
            '<div style="font-size:0.85em;color:#888;margin:6px 0">Author: ' + escapeHtml(u.author || "system") + ' &mdash; ' + fmtDate(u.created_at) + '</div>' +
            '<div style="margin-bottom:8px;font-size:0.9em">' + escapeHtml(u.description || "") + '</div>' +
            '<details style="margin-top:8px"><summary style="cursor:pointer;color:var(--accent)">View Changes (' + (u.changes || []).length + ')</summary>' + changesHtml + '</details>' +
            '<div style="margin-top:8px">' + commentsHtml + '</div>' +
            '<div style="margin-top:12px">' + actions + '</div>';

        grid.appendChild(card);
    });
    container.appendChild(grid);

    // Pagination
    if (pages > 1) {
        var pager = document.createElement("div");
        pager.style.cssText = "display:flex;justify-content:center;align-items:center;gap:8px;margin-top:12px";
        var prevBtn = document.createElement("button"); prevBtn.className = "btn"; prevBtn.textContent = "Prev"; prevBtn.disabled = currentPage <= 1;
        prevBtn.onclick = function() { currentPage = Math.max(1, currentPage - 1); renderUpdates(); };
        var nextBtn = document.createElement("button"); nextBtn.className = "btn"; nextBtn.textContent = "Next"; nextBtn.disabled = currentPage >= pages;
        nextBtn.onclick = function() { currentPage = Math.min(pages, currentPage + 1); renderUpdates(); };
        var label = document.createElement("div"); label.className = "muted"; label.textContent = "Page " + currentPage + " / " + pages;
        pager.appendChild(prevBtn); pager.appendChild(label); pager.appendChild(nextBtn);
        container.appendChild(pager);
    }
}

// ── Actions ────────────────────────────────────────────────
async function brainstormUpdate() {
    setStatus("Thinking...");
    try {
        const res = await safeFetch("/api/admin/evolution/think", { method: "POST", credentials: "same-origin", headers: authHeaders() });
        if (!res.ok) { setStatus(await parseErr(res, "Failed to brainstorm."), true); return; }
        const p = await res.json();
        setStatus((p.message || "Think cycle complete.") + (p.reason ? " (" + p.reason + ")" : ""));
        await fullRefresh();
    } catch (e) { setStatus("Error: " + e.message, true); }
}

async function applyUpdate(id) {
    if (!confirm("Apply this update?")) return;
    const res = await safeFetch("/api/admin/evolution/apply/" + id, { method: "POST", credentials: "same-origin", headers: authHeaders() });
    if (!res.ok) { setStatus(await parseErr(res, "Failed to apply."), true); return; }
    setStatus("Update applied."); await fullRefresh();
}

async function denyUpdate(id) {
    if (!confirm("Deny this update?")) return;
    const res = await safeFetch("/api/admin/evolution/deny/" + id, { method: "POST", credentials: "same-origin", headers: authHeaders() });
    if (!res.ok) { setStatus(await parseErr(res, "Failed to deny."), true); return; }
    setStatus("Update denied."); await fullRefresh();
}

async function resolveUpdate(id) {
    if (!confirm("Resolve this update?")) return;
    const res = await safeFetch("/api/admin/evolution/resolve/" + id, { method: "POST", credentials: "same-origin", headers: authHeaders() });
    if (!res.ok) { setStatus(await parseErr(res, "Failed to resolve."), true); return; }
    setStatus("Update resolved."); await fullRefresh();
}

async function rollbackUpdate(id) {
    if (!confirm("Rollback this update?")) return;
    const res = await safeFetch("/api/admin/evolution/rollback/" + id, { method: "POST", credentials: "same-origin", headers: authHeaders() });
    if (!res.ok) { setStatus(await parseErr(res, "Failed to rollback."), true); return; }
    setStatus("Update rolled back."); await fullRefresh();
}

async function voteUpdate(id, vote) {
    try {
        const res = await safeFetch("/api/evolution/vote/" + encodeURIComponent(id), {
            method: "POST", credentials: "same-origin",
            headers: authHeaders(true),
            body: JSON.stringify({ vote: vote }),
        });
        if (!res.ok) { setStatus(await parseErr(res, "Failed to vote."), true); return; }
        const p = await res.json();
        var u = allUpdates.find(function(x) { return x.id === id; });
        if (u) { u.votes_up = p.votes_up ?? u.votes_up ?? 0; u.votes_down = p.votes_down ?? u.votes_down ?? 0; }
        renderUpdates();
    } catch (e) { setStatus("Vote failed.", true); }
}

// ── Training ───────────────────────────────────────────────
async function setTrainingMode(enabled) {
    try {
        const res = await safeFetch("/api/admin/training/mode", {
            method: "POST", credentials: "same-origin",
            headers: authHeaders(true),
            body: JSON.stringify({ enabled: enabled }),
        });
        if (!res.ok) { setStatus(await parseErr(res, "Failed to update training."), true); return; }
        const p = await res.json();
        if (enabled) {
            var nw = p.report ? (p.report.nodes_written || 0) : 0;
            var ws = p.report ? ((p.report.websites_scraped || []).length) : 0;
            var dur = p.report ? (p.report.duration_ms || 0) : 0;
            setStatus("Training started. " + ws + " site(s), " + nw + " node(s), " + fmtMs(dur));
        } else {
            setStatus("Training stopped.");
        }
        await loadTrainingStatus();
    } catch (e) { setStatus("Training error: " + e.message, true); }
}

async function runTrainingNow() {
    setStatus("Running training cycle...");
    try {
        const res = await safeFetch("/api/admin/training/run", { method: "POST", credentials: "same-origin", headers: authHeaders() });
        if (!res.ok) { setStatus(await parseErr(res, "Training failed."), true); return; }
        const p = await res.json();
        var nw = p.report ? (p.report.nodes_written || 0) : 0;
        var dur = p.report ? (p.report.duration_ms || 0) : 0;
        setStatus("Training done. " + nw + " node(s) written in " + fmtMs(dur));
        await loadBrainStats();
        await loadTrainingStatus();
    } catch (e) { setStatus("Training error: " + e.message, true); }
}

async function loadTrainingStatus() {
    if (!rootAdmin) return;
    try {
        const res = await safeFetch("/api/admin/training/status", { credentials: "same-origin", headers: authHeaders() });
        if (!res.ok) { setText("training-status", "Could not load training status."); return; }
        const data = await res.json();
        var tr = data.training || {};
        var parts = [];
        parts.push("Enabled: " + (tr.enabled ? "yes" : "no"));
        parts.push("Internet: " + (data.internet_enabled ? "on" : "off"));
        parts.push("Cycles: " + (tr.total_cycles || 0));
        parts.push("Nodes written: " + (tr.total_nodes_written || 0));
        setText("training-status", parts.join(" | "));
    } catch (e) { setText("training-status", "Error loading training."); }
}

// ── Sessions ───────────────────────────────────────────────
async function loadActiveSessions() {
    var container = el("active-sessions");
    if (!container) return;
    try {
        const res = await safeFetch("/api/admin/sessions", { credentials: "same-origin", headers: authHeaders() });
        if (!res.ok) {
            container.innerHTML = '<div class="muted">' + (res.status === 403 ? "Sign in as admin to view sessions." : "Could not load sessions.") + '</div>';
            return;
        }
        const sessions = await res.json();
        if (!Array.isArray(sessions) || sessions.length === 0) {
            container.innerHTML = '<div class="muted">No active sessions.</div>';
            return;
        }
        container.innerHTML = '<div style="max-height:300px;overflow:auto">' + sessions.map(function(s) {
            return '<div class="session-row" style="justify-content:space-between"><div style="flex:1;min-width:0"><div><strong>' + escapeHtml(s.username) + '</strong> <span class="muted">' + escapeHtml(s.ip) + '</span></div><div class="muted" style="font-size:0.8em;margin-top:2px">Last: ' + fmtDate(s.last_seen) + '</div></div><button class="btn" style="flex-shrink:0" onclick="terminateSession(\'' + escapeHtml(s.username) + '\')">End</button></div>';
        }).join("") + '</div>';
    } catch (e) { container.innerHTML = '<div class="muted">Error loading sessions.</div>'; }
}

async function terminateSession(username) {
    if (!confirm("Terminate session for " + username + "?")) return;
    await safeFetch("/api/admin/session/" + encodeURIComponent(username), { method: "DELETE", credentials: "same-origin", headers: authHeaders() });
    await loadActiveSessions();
}

// ── Full Refresh ───────────────────────────────────────────
async function fullRefresh() {
    await Promise.all([loadBrainStats(), loadUpdates()]);
    await loadActiveSessions();
    if (rootAdmin) await loadTrainingStatus();
}

// ── Initialize ─────────────────────────────────────────────
(async function() {
    console.log("[Evolution] Initializing...");

    // 1. Load brain stats immediately (no auth needed)
    loadBrainStats();

    // 2. Check auth
    try {
        var auth = await getAuthState();
        showAuthBanner(auth);
        userRole = (auth.isAdmin && auth.username === ROOT_ADMIN) ? "admin" : (auth.loggedIn ? "user" : "guest");
        rootAdmin = (userRole === "admin");
        if (rootAdmin) {
            startAuthGuard("admin", 60000);
            var ac = el("admin-controls-card");
            if (ac) ac.style.display = "";
        }
        console.log("[Evolution] Auth:", userRole, "admin:", rootAdmin);
    } catch (e) {
        console.warn("[Evolution] Auth check failed:", e);
        showAuthBanner(null);
    }

    // 3. Load data
    await loadUpdates();
    await loadBrainStats();  // reload after auth (may get richer data)
    await loadActiveSessions();
    if (rootAdmin) await loadTrainingStatus();

    // 4. Auto-refresh
    setInterval(function() { loadBrainStats(); }, 15000);
    setInterval(function() { loadActiveSessions(); }, 20000);
    setInterval(function() { loadUpdates(); }, 30000);

    console.log("[Evolution] Ready. Updates:", allUpdates.length);
})();
