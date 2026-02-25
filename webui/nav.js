/**
 * JeebsAI — Sidebar Navigation Component
 * Include after auth.js on every page.
 *
 * Usage: <aside id="sidebar-container" data-active="home"></aside>
 * Then call: JeebsNav.init('home');
 */
const JeebsNav = (function () {
    const LOGO_SVG = `<svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="16" cy="16" r="14" stroke="currentColor" stroke-width="2"/><circle cx="16" cy="16" r="6" fill="currentColor" opacity=".6"/><path d="M16 2v8M16 22v8M2 16h8M22 16h8" stroke="currentColor" stroke-width="1.5" opacity=".4"/></svg>`;

    // General navigation
    const PAGES = [
        { id: 'home', label: 'Console', href: '/webui/index.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'Reguser', 'Guest'] },
        { id: 'profile', label: 'Profile', href: '/webui/profile.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'Reguser'] },
        { id: 'search', label: 'Brain Search', href: '/webui/search.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'Reguser'] },
        { id: 'status', label: 'Status', href: '/webui/status.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer'] },
    ];

    let currentActiveId = 'home';

    // TrustedHTML bypass policy for security-hardened browsers
    let policy = { createHTML: (s) => s };
    if (window.trustedTypes && window.trustedTypes.createPolicy) {
        try {
            policy = window.trustedTypes.createPolicy('jeebs-nav-policy', {
                createHTML: (string) => string
            });
        } catch (e) {
            console.warn("TrustedTypes policy creation failed:", e);
        }
    }

    // Admin/Trainer/Advanced navigation
    const ADMIN_PAGES = [
        { id: 'admin', label: 'Admin', href: '/webui/admin_dashboard.html', roles: ['Admin', 'super_admin'] },
        { id: 'users', label: 'Users', href: '/webui/admin_users.html', roles: ['Admin', 'super_admin'] },
        { id: 'trainer', label: 'Trainer', href: '/webui/trainer_panel.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'trainer'] },
        { id: 'resources', label: 'Resources', href: '/webui/trainer_resources.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'trainer'] },
        { id: 'logs', label: 'Logs', href: '/webui/logs.html', roles: ['Admin', 'super_admin', 'Developer'] },
        { id: 'evolution', label: 'Evolution', href: '/webui/evolution.html', roles: ['Admin', 'super_admin', 'Developer'] },
        { id: 'brain', label: 'Brain Graph', href: '/webui/visualize.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'trainer'] },
        { id: 'logic', label: 'Logic Graph', href: '/webui/logic_visualize.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'trainer'] },
        { id: 'blacklist', label: 'Blacklist', href: '/webui/admin_blacklist.html', roles: ['Admin', 'super_admin', 'Mod'] },
        { id: 'whitelist', label: 'Whitelist', href: '/webui/admin_whitelist.html', roles: ['Admin', 'super_admin', 'Mod'] },
        { id: 'anomalies', label: 'Anomalies', href: '/webui/admin_anomalies.html', roles: ['Admin', 'super_admin'] },
        { id: 'reasoning', label: 'Reasoning', href: '/webui/admin_reasoning.html', roles: ['Admin', 'super_admin'] },
        { id: 'thoughts', label: 'Thoughts', href: '/webui/thought_monitor.html', roles: ['Admin', 'super_admin', 'Mod', 'Trainer', 'trainer'] },
    ];

    function render(activeId) {
        const container = document.getElementById('sidebar-container');
        if (!container) return;

        // Determine admin/root state robustly (works even if auth.js wasn't loaded)
        const storedUsername = localStorage.getItem('jeebs_username') || '';
        let userRole = localStorage.getItem('jeebs_role') || 'Guest';
        let isAdmin = localStorage.getItem('jeebs_is_admin') === 'true';

        // Hardcode root admin role check, just in case
        if (storedUsername === '1090mb') {
            userRole = 'super_admin';
            isAdmin = true;
        } else if (isAdmin && userRole === 'Reguser') {
            userRole = 'Admin';
        }

        // Helper to check if current role is allowed for a page
        const isAllowed = (page) => {
            if (storedUsername === 'peaci' && page.id === 'trainer') return true;
            if (page.roles.includes(userRole)) return true;
            // Admins should see everything a regular user, trainer, or developer sees
            if (isAdmin && (page.roles.includes('Reguser') || page.roles.includes('Trainer') || page.roles.includes('trainer') || page.roles.includes('Developer'))) return true;
            return false;
        };

        let linksHtml = PAGES.filter(function (p) {
            return isAllowed(p);
        }).map(function (p) {
            return `<a class="sidebar-link${p.id === activeId ? ' active' : ''}" href="${p.href}">${p.label}</a>`;
        }).join('');

        // Add admin/advanced links if allowed
        linksHtml += ADMIN_PAGES.filter(function (p) {
            return isAllowed(p);
        }).map(function (p) {
            return `<a class="sidebar-link${p.id === activeId ? ' active' : ''}" href="${p.href}">${p.label}</a>`;
        }).join('');

        if (userRole === 'Guest') {
            linksHtml += `<a class="sidebar-link" href="/webui/index.html#login" style="margin-top: auto; border-top: 1px solid var(--border); padding-top: 20px; color: var(--accent);">Sign In / Register</a>`;
        }

        container.className = 'sidebar';
        container.innerHTML = policy.createHTML(`
            <div class="sidebar-header">
                ${LOGO_SVG}
                <span>JeebsAI</span>
                <span id="jeebs-version" class="jeebs-version">v...</span>
            </div>
            <nav class="sidebar-links" id="sidebarLinks">
                ${linksHtml}
            </nav>
            <div class="sidebar-footer">
                <div class="sidebar-status" id="navStatus">
                    <span class="dot" id="navDot"></span>
                    <span id="navStatusText">Checking...</span>
                </div>
            </div>
        `);

        // Inject Mobile Navbar globally if not already present
        if (!document.getElementById('mobileNavHeader')) {
            const mobileHeader = document.createElement('div');
            mobileHeader.id = 'mobileNavHeader';
            mobileHeader.className = 'mobile-nav-header';
            mobileHeader.innerHTML = `
                <div class="mobile-brand">
                    ${LOGO_SVG}
                    <span>JeebsAI</span>
                </div>
                <button class="mobile-toggle-btn" id="mobileToggleBtn">&#9776;</button>
            `;
            document.body.prepend(mobileHeader);

            document.getElementById('mobileToggleBtn').addEventListener('click', () => {
                container.classList.toggle('open');
            });

            // Close mobile nav when clicking a link
            container.querySelectorAll('a').forEach(function (a) {
                a.addEventListener('click', function () {
                    container.classList.remove('open');
                });
            });
        }

        // Check auth state for status indicator
        checkStatus();
        // Fetch current server-side version (non-blocking)
        fetchVersion();
    }

    async function fetchVersion() {
        try {
            const res = await fetch('/api/version', { credentials: 'same-origin' });
            if (!res.ok) return;
            const data = await res.json();
            const el = document.getElementById('jeebs-version');
            if (el && data && data.version) {
                // Ensure a leading 'v' for display consistency
                el.textContent = data.version.startsWith('v') ? data.version : ('v' + data.version);
            }
        } catch (e) { /* ignore */ }
    }

    async function checkStatus() {
        const prevRole = localStorage.getItem('jeebs_role');
        const dot = document.getElementById('navDot');
        const text = document.getElementById('navStatusText');
        if (!dot || !text) return;

        try {
            const auth = await getAuthState();
            if (auth.loggedIn) {
                dot.className = 'dot online';
                // Try to fetch richer session info for debugging
                try {
                    const res = await fetch('/api/auth/session', { credentials: 'same-origin' });
                    if (res.ok) {
                        const body = await res.json();
                        // /api/auth/session returns either { session: {...} } or { identity: {...} }
                        const info = body.session || body.identity || {};
                        const username = info.username || auth.username || 'unknown';
                        const role = info.role || auth.role || 'Reguser';
                        const badges = [];
                        if (info.is_admin || auth.isAdmin) badges.push('admin');
                        if (info.is_trainer || auth.isTrainer) badges.push('trainer');
                        text.textContent = username + (badges.length ? ' (' + badges.join(',') + ')' : '');
                        localStorage.setItem('jeebs_is_admin', (info.is_admin || auth.isAdmin) ? 'true' : 'false');
                        localStorage.setItem('jeebs_username', username || '');
                        localStorage.setItem('jeebs_role', role || 'Reguser');
                    } else {
                        text.textContent = auth.username || 'Signed in';
                    }
                } catch (e) {
                    text.textContent = auth.username || 'Signed in';
                }
            } else {
                dot.className = 'dot offline';
                text.textContent = 'Not signed in';
                localStorage.removeItem('jeebs_is_admin');
                localStorage.removeItem('jeebs_username');
                localStorage.removeItem('jeebs_role');
            }

            // If the role changed significantly (from Guest to something else), re-render the link list
            if (prevRole !== localStorage.getItem('jeebs_role')) {
                render(currentActiveId);
            }
        } catch (e) {
            dot.className = 'dot offline';
            text.textContent = 'Offline';
        }
    }

    // Theme switching
    const themes = ['dark', 'light', 'neon'];
    let currentTheme = localStorage.getItem('jeebs-theme') || 'dark';
    document.body.setAttribute('data-theme', currentTheme);

    function updateThemeButton() {
        const btn = document.getElementById('themeBtn');
        if (btn) {
            btn.textContent = `Theme: ${currentTheme.charAt(0).toUpperCase() + currentTheme.slice(1)}`;
        }
    }
    updateThemeButton();

    document.addEventListener('click', function (e) {
        if (e.target.id === 'themeBtn') {
            const nextIndex = (themes.indexOf(currentTheme) + 1) % themes.length;
            currentTheme = themes[nextIndex];
            document.body.setAttribute('data-theme', currentTheme);
            localStorage.setItem('jeebs-theme', currentTheme);
            updateThemeButton();
        }
    });

    return {
        init: function (activeId) {
            currentActiveId = activeId || 'home';
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', function () { render(currentActiveId); });
            } else {
                render(currentActiveId);
            }
        },
        refresh: checkStatus,
    };
})();
