/**
 * JeebsAI shared authentication & fetch utilities.
 * Include this in every page: <script src="/webui/auth.js"></script>
 *
 * Provides:
 *  - Token persistence (localStorage)
 *  - authHeaders()        — returns headers with Bearer token
 *  - safeFetch(url, opts) — fetch with 429 back-off retry
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

/* ── safeFetch with 429 back-off ─────────────────────── */

async function safeFetch(url, options, retries) {
    // allow more retries and progressively longer delay
    if (retries === undefined) retries = 4;
    const res = await fetch(url, options);
    if (res.status === 429 && retries > 0) {
        const delay = 1000 + (5 - retries) * 500; // 1s,1.5s,2s,...
        await new Promise(function (r) { setTimeout(r, delay); });
        return safeFetch(url, options, retries - 1);
    }
    return res;
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

    // If not logged in but we have a token locally, try a lightweight session ping
    if (!auth.loggedIn && jeebsGetToken()) {
        try {
            const pingRes = await safeFetch("/api/session/ping", { method: "POST", headers: authHeaders() });
            if (pingRes.ok) {
                // refresh auth state
                auth = await getAuthState();
            }
        } catch (e) {
            // network issue; fall through to redirect below
        }
    }

    if (!auth.loggedIn) {
        window.location.replace("/webui/index.html");
        return null;
    }

    if (role === "admin") {
        if (!auth.isAdmin || auth.username !== JEEBS_ROOT_ADMIN) {
            window.location.replace("/webui/index.html");
            return null;
        }
    } else if (role === "trainer") {
        if (!auth.isTrainer && !(auth.isAdmin && auth.username === JEEBS_ROOT_ADMIN)) {
            window.location.replace("/webui/index.html");
            return null;
        }
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

function startAuthGuard(role, intervalMs) {
    // default to a longer interval to avoid transient redirects (5 minutes)
    if (!intervalMs) intervalMs = 300000;

    // Re-check token validity in background
    setInterval(async function () {
        var auth = await getAuthState();
        if (!auth.loggedIn) {
            jeebsClearToken();
            // Session expired; do not auto-redirect from admin/trainer panels.
            // Show a non-blocking warning instead so the user can re-auth without losing context.
            try { alert("Session expired — please re-authenticate to continue."); } catch(e) { console.warn("session expired"); }
            return;
        }
        if (role === "admin" && (!auth.isAdmin || auth.username !== JEEBS_ROOT_ADMIN)) {
            try { alert("Admin privileges revoked — please re-authenticate."); } catch(e) { console.warn("admin revoked"); }
        }
    }, intervalMs);

    // Re-check when tab becomes visible
    document.addEventListener("visibilitychange", async function () {
        if (document.hidden) return;
        var auth = await getAuthState();
        if (!auth.loggedIn) {
            jeebsClearToken();
            try { alert("Session expired — please re-authenticate to continue."); } catch(e) { console.warn("session expired"); }
        }
    });

    // Watch for token removal in another tab
    window.addEventListener("storage", function (e) {
        if (e.key === JEEBS_TOKEN_KEY && !e.newValue) {
            try { console.warn("Auth token removed in another tab"); } catch(e) {}
        }
    });
}

/* ── session ping ────────────────────────────────────── */

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
