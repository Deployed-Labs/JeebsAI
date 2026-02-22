/**
 * JeebsAI shared authentication & fetch utilities.
 * Include this in every page: <script src="/webui/auth.js"></script>
 *
 * Provides:
 *  - Token persistence (localStorage)
 *  - authHeaders()        — returns headers with Bearer token
 *  - safeFetch(url, opts) — fetch wrapper (no rate-limit retry)
 *  - requireAuth(role)    — guard: redirects to /webui/index.html if not authorised
 *  - getAuthState()       — returns { loggedIn, username, isAdmin, isTrainer }
 *  - logout()             — clears token and redirects
 */

const JEEBS_TOKEN_KEY = "jeebs_auth_token";
const JEEBS_ROOT_ADMIN = "1090mb";

/* ── helpers ─────────────────────────────────────────── */

function jeebsGetToken() {
    return localStorage.getItem(JEEBS_TOKEN_KEY) || "";
}

function jeebsSetToken(token) {
    if (token) localStorage.setItem(JEEBS_TOKEN_KEY, token);
}

function jeebsClearToken() {
    localStorage.removeItem(JEEBS_TOKEN_KEY);
}

/* ── auth headers ────────────────────────────────────── */

function authHeaders(json = false) {
    const h = json ? { "Content-Type": "application/json" } : {};
    const token = jeebsGetToken();
    if (token) h["Authorization"] = "Bearer " + token;
    return h;
}

/* ── safeFetch — simple fetch wrapper (no rate-limit logic) ── */

async function safeFetch(url, options) {
    // Harden fetch: always use POST for sensitive auth, and check for CSRF
    if (options && options.method && options.method.toUpperCase() === 'POST') {
        if (!options.headers) options.headers = {};
        options.headers['X-Requested-With'] = 'XMLHttpRequest';
    }
    return fetch(url, options);
}

/* ── auth state ──────────────────────────────────────── */

async function getAuthState() {
    try {
        const res = await safeFetch("/api/auth/status", {
            headers: authHeaders(),
            credentials: "same-origin",
        });
        if (!res.ok) return { loggedIn: false, username: "", isAdmin: false, isTrainer: false };
        const data = await res.json();
        if (data.token) jeebsSetToken(data.token);
        return {
            loggedIn: !!data.logged_in,
            username: data.username || "",
            isAdmin: !!data.is_admin,
            isTrainer: !!data.is_trainer,
        };
    } catch (_) {
        return { loggedIn: false, username: "", isAdmin: false, isTrainer: false };
    }
}

/**
 * Guard function — call at page load.
 *  role = "admin"   → must be root admin (1090mb)
 *  role = "trainer" → must be trainer or root admin
 *  role = "user"    → must be logged in
 *  Returns auth state if authorised; redirects otherwise.
 */
async function requireAuth(role) {
    let auth = await getAuthState();

    // If not logged in and no token, restrict access
    if (!auth.loggedIn && !jeebsGetToken()) {
        // If not on index.html, redirect to register
        const path = window.location.pathname;
        if (!path.endsWith('index.html')) {
            window.location.replace('/webui/index.html#register');
        }
    }

    // If not logged in but we have a token locally, try a lightweight session ping
    if (!auth.loggedIn && jeebsGetToken()) {
        try {
            const pingRes = await safeFetch("/api/session/ping", { method: "POST", headers: authHeaders() });
            if (pingRes.ok) {
                auth = await getAuthState();
            }
        } catch (e) {}
    }
    return auth;
}

/* ── logout ──────────────────────────────────────────── */

async function logout() {
    try {
        await safeFetch("/api/logout", { method: "POST", headers: authHeaders() });
    } catch (_) { /* ignore */ }
    jeebsClearToken();
    window.location.replace("/webui/index.html");
}

/* ── periodic auth guard ─────────────────────────────── */

// Track guard state so repeated page loads don't attach multiple timers/listeners
var _jeebsAuthGuardTimer = null;
var _jeebsAuthGuardRole = null;
var _jeebsAuthGuardVisibilityHandler = null;
var _jeebsAuthGuardStorageHandler = null;

function startAuthGuard(role, intervalMs) {
    // default to a longer interval to avoid transient redirects (5 minutes)
    if (!intervalMs) intervalMs = 300000;

    // If guard already started for this role, don't attach again
    if (_jeebsAuthGuardTimer && _jeebsAuthGuardRole === role) return;

    // clear any existing guard first
    stopAuthGuard();

    _jeebsAuthGuardRole = role;
    // Re-check token validity in background
    _jeebsAuthGuardTimer = setInterval(async function () {
        var auth = await getAuthState();
        if (!auth.loggedIn) {
            // Session appears invalid/expired. Do NOT clear the local token or redirect.
            // Instead notify the user so they can choose to renew or expire manually.
            try { console.warn("Session appears expired — manual re-auth recommended."); } catch(e) { }
            try { showSessionBanner(); } catch(e) {}
            return;
        }
        if (role === "admin" && (!auth.isAdmin || auth.username !== JEEBS_ROOT_ADMIN)) {
            try { console.warn("Admin privileges changed — manual re-auth recommended."); } catch(e) { }
        }
    }, intervalMs);

    // Re-check when tab becomes visible
    if (_jeebsAuthGuardVisibilityHandler) document.removeEventListener('visibilitychange', _jeebsAuthGuardVisibilityHandler);
    _jeebsAuthGuardVisibilityHandler = async function () {
        if (document.hidden) return;
        var auth = await getAuthState();
        if (!auth.loggedIn) {
            try { console.warn("Session expired (visibility change) — manual re-auth recommended."); } catch(e) {}
            try { showSessionBanner(); } catch(e) {}
        }
    };
    document.addEventListener("visibilitychange", _jeebsAuthGuardVisibilityHandler);

    // Watch for token removal in another tab
    if (_jeebsAuthGuardStorageHandler) window.removeEventListener('storage', _jeebsAuthGuardStorageHandler);
    _jeebsAuthGuardStorageHandler = function (e) {
        if (e.key === JEEBS_TOKEN_KEY && !e.newValue) {
            try { console.warn("Auth token removed in another tab"); } catch(e) {}
        }
    };
    window.addEventListener("storage", _jeebsAuthGuardStorageHandler);
}

function stopAuthGuard() {
    try {
        if (_jeebsAuthGuardTimer) { clearInterval(_jeebsAuthGuardTimer); _jeebsAuthGuardTimer = null; }
        if (_jeebsAuthGuardVisibilityHandler) { document.removeEventListener('visibilitychange', _jeebsAuthGuardVisibilityHandler); _jeebsAuthGuardVisibilityHandler = null; }
        if (_jeebsAuthGuardStorageHandler) { window.removeEventListener('storage', _jeebsAuthGuardStorageHandler); _jeebsAuthGuardStorageHandler = null; }
        _jeebsAuthGuardRole = null;
    } catch (e) { /* ignore */ }
}

/* ── session ping ────────────────────────────────────── */

// Token control UI: show masked token and provide an explicit "Expire token" button
function renderTokenControl() {
    try {
        const existing = document.getElementById('token-control');
        if (existing) return;
        const box = document.createElement('div');
        box.id = 'token-control';
        box.style.position = 'fixed';
        box.style.right = '12px';
        box.style.bottom = '12px';
        box.style.background = 'rgba(20,22,28,0.95)';
        box.style.border = '1px solid var(--border)';
        box.style.color = 'var(--text)';
        box.style.padding = '10px';
        box.style.borderRadius = '8px';
        box.style.zIndex = 9999;
        box.style.fontSize = '13px';

        function maskedToken() {
            const t = jeebsGetToken() || '';
            if (!t) return '(no token)';
            return t.slice(0,6) + '…' + t.slice(-6);
        }

        box.innerHTML = `<div style="display:flex;gap:8px;align-items:center;"><div id="token-display" style="opacity:0.9">Token: ${maskedToken()}</div><button id="expire-token-btn" style="margin-left:8px;padding:6px 8px;border-radius:6px;border:1px solid var(--border);background:#2b2f3a;color:var(--text);cursor:pointer;">Expire</button></div>`;
        document.body.appendChild(box);

        document.getElementById('expire-token-btn').addEventListener('click', function () {
            if (!confirm('Expire your token now? This will log you out.')) return;
            // logout() clears token and redirects
            logout();
        });

        // update display periodically
        setInterval(() => {
            const el = document.getElementById('token-display');
            if (el) el.textContent = 'Token: ' + maskedToken();
        }, 5000);
    } catch (e) { console.warn('renderTokenControl failed', e); }
}

// Auto-render token control on load
if (typeof window !== 'undefined') {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', renderTokenControl);
    } else {
        renderTokenControl();
    }
}

// Session expired banner: shows a non-blocking banner prompting re-auth or expire
function renderSessionBanner() {
    try {
        if (document.getElementById('session-expired-banner')) return;
        const b = document.createElement('div');
        b.id = 'session-expired-banner';
        b.style.position = 'fixed';
        b.style.left = '12px';
        b.style.right = '12px';
        b.style.top = '12px';
        b.style.zIndex = 10000;
        b.style.padding = '10px 12px';
        b.style.borderRadius = '8px';
        b.style.display = 'none';
        b.style.alignItems = 'center';
        b.style.justifyContent = 'space-between';
        b.style.background = '#2f2f36';
        b.style.border = '1px solid rgba(255,200,120,0.08)';
        b.style.color = 'var(--text)';

        b.innerHTML = `<div style="display:flex;gap:12px;align-items:center"><strong>Session appears expired</strong><span class="muted" style="font-size:0.95em">Your server session may be invalid. You can re-authenticate or expire the local token.</span></div><div style="display:flex;gap:8px"><button id="reopen-login-btn" class="btn">Re-auth</button><button id="expire-now-btn" class="btn danger">Expire</button></div>`;
        document.body.appendChild(b);

        document.getElementById('reopen-login-btn').addEventListener('click', function () {
            // open login page in same tab
            window.location.href = '/webui/index.html';
        });
        document.getElementById('expire-now-btn').addEventListener('click', function () {
            if (!confirm('Expire local token and logout?')) return;
            logout();
        });
    } catch (e) { console.warn('renderSessionBanner failed', e); }
}

function showSessionBanner() {
    try {
        renderSessionBanner();
        const b = document.getElementById('session-expired-banner');
        if (b) b.style.display = 'flex';
    } catch (e) { }
}

function hideSessionBanner() {
    try { const b = document.getElementById('session-expired-banner'); if (b) b.style.display = 'none'; } catch(e) {}
}

var _jeebsPingTimer = null;

function startSessionPing(intervalMs) {
    if (_jeebsPingTimer) return;
    // ping every 60s by default to keep server-side session fresh
    if (!intervalMs) intervalMs = 60000;
    _jeebsPingTimer = setInterval(function () {
        safeFetch("/api/session/ping", { method: "POST", headers: authHeaders() }).catch(function () {});
    }, intervalMs);
    safeFetch("/api/session/ping", { method: "POST", headers: authHeaders() }).catch(function () {});
}

function stopSessionPing() {
    if (_jeebsPingTimer) { clearInterval(_jeebsPingTimer); _jeebsPingTimer = null; }
}
