/**
 * JeebsAI — Shared Navigation Component
 * Include after auth.js on every page.
 *
 * Usage: <nav id="topnav" data-active="home"></nav>
 * Then call: JeebsNav.init('home');
 */
const JeebsNav = (function () {
    const LOGO_SVG = `<svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="16" cy="16" r="14" stroke="currentColor" stroke-width="2"/><circle cx="16" cy="16" r="6" fill="currentColor" opacity=".6"/><path d="M16 2v8M16 22v8M2 16h8M22 16h8" stroke="currentColor" stroke-width="1.5" opacity=".4"/></svg>`;

    // General navigation
    const PAGES = [
        { id: 'home', label: 'Console', href: '/webui/index.html', roles: ['Admin', 'Mod', 'Trainer', 'Reguser'] },
        { id: 'profile', label: 'Profile', href: '/webui/profile.html', roles: ['Admin', 'Mod', 'Trainer', 'Reguser'] },
        { id: 'search', label: 'Brain Search', href: '/webui/search.html', roles: ['Admin', 'Mod', 'Trainer', 'Reguser'] },
        { id: 'status', label: 'Status', href: '/webui/status.html', roles: ['Admin', 'Mod', 'Trainer'] },
    ];

    // Admin/Trainer/Advanced navigation
    const ADMIN_PAGES = [
        { id: 'admin', label: 'Admin', href: '/webui/admin_dashboard.html', roles: ['Admin'] },
        { id: 'users', label: 'Users', href: '/webui/admin_users.html', roles: ['Admin'] },
        { id: 'trainer', label: 'Trainer', href: '/webui/trainer_panel.html', roles: ['Admin', 'Mod', 'Trainer'] },
        { id: 'resources', label: 'Resources', href: '/webui/trainer_resources.html', roles: ['Admin', 'Mod', 'Trainer'] },
        { id: 'logs', label: 'Logs', href: '/webui/logs.html', roles: ['Admin'] },
        { id: 'evolution', label: 'Evolution', href: '/webui/evolution.html', roles: ['Admin'] },
        { id: 'brain', label: 'Brain Graph', href: '/webui/visualize.html', roles: ['Admin', 'Mod', 'Trainer'] },
        { id: 'logic', label: 'Logic Graph', href: '/webui/logic_visualize.html', roles: ['Admin', 'Mod', 'Trainer'] },
        { id: 'blacklist', label: 'Blacklist', href: '/webui/admin_blacklist.html', roles: ['Admin', 'Mod'] },
        { id: 'whitelist', label: 'Whitelist', href: '/webui/admin_whitelist.html', roles: ['Admin', 'Mod'] },
        { id: 'anomalies', label: 'Anomalies', href: '/webui/admin_anomalies.html', roles: ['Admin'] },
        { id: 'reasoning', label: 'Reasoning', href: '/webui/admin_reasoning.html', roles: ['Admin'] },
    ];

    function render(activeId) {
        const container = document.getElementById('topnav');
        if (!container) return;

        // Determine admin/root state robustly (works even if auth.js wasn't loaded)
        const storedUsername = localStorage.getItem('jeebs_username') || '';
        let userRole = localStorage.getItem('jeebs_role') || 'Guest';
        
        // Hardcode root admin role check
        if (storedUsername === '1090mb') {
            userRole = 'Admin';
        }

        // Helper to check if current role is allowed for a page
        const isAllowed = (page) => page.roles && page.roles.includes(userRole);

        let linksHtml = PAGES.filter(function (p) {
            return isAllowed(p);
        }).map(function (p) {
            return `<a class="topnav-link${p.id === activeId ? ' active' : ''}" href="${p.href}">${p.label}</a>`;
        }).join('');

        // Add admin/advanced links if allowed
        linksHtml += ADMIN_PAGES.filter(function (p) {
            return isAllowed(p);
        }).map(function (p) {
            return `<a class="topnav-link${p.id === activeId ? ' active' : ''}" href="${p.href}">${p.label}</a>`;
        }).join('');

        container.className = 'topnav';
        container.innerHTML = `
            <div class="topnav-inner">
                <a class="topnav-brand" href="/webui/index.html">
                    ${LOGO_SVG}
                    <span>JeebsAI <span id="jeebs-version" class="jeebs-version">v0.0.1</span></span>
                </a>
                <button class="topnav-toggle" id="navToggle" aria-label="Toggle navigation">&#9776;</button>
                <div class="topnav-links" id="navLinks">
                    ${linksHtml}
                </div>
                <div class="topnav-status" id="navStatus">
                    <span class="dot" id="navDot"></span>
                    <span id="navStatusText">Checking...</span>
                </div>
                <button class="theme-btn" id="themeBtn">Theme: Dark</button>
            </div>
        `;

        // Mobile toggle
        const toggle = document.getElementById('navToggle');
        const links = document.getElementById('navLinks');
        if (toggle && links) {
            toggle.addEventListener('click', function () {
                links.classList.toggle('open');
            });
            // Close mobile nav when clicking a link
            links.querySelectorAll('a').forEach(function (a) {
                a.addEventListener('click', function () {
                    links.classList.remove('open');
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
            if (el && data && data.version) el.textContent = data.version;
        } catch (e) { /* ignore */ }
    }

    async function checkStatus() {
        const dot = document.getElementById('navDot');
        const text = document.getElementById('navStatusText');
        if (!dot || !text) return;

        try {
            const auth = await getAuthState();
            if (auth.loggedIn) {
                dot.className = 'dot online';
                text.textContent = auth.username;
                localStorage.setItem('jeebs_is_admin', auth.isAdmin ? 'true' : 'false');
                localStorage.setItem('jeebs_username', auth.username || '');
                localStorage.setItem('jeebs_role', auth.role || 'Reguser');
            } else {
                dot.className = 'dot offline';
                text.textContent = 'Not signed in';
                localStorage.removeItem('jeebs_is_admin');
                localStorage.removeItem('jeebs_username');
                localStorage.removeItem('jeebs_role');
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

    document.addEventListener('click', function(e) {
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
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', function () { render(activeId); });
            } else {
                render(activeId);
            }
        },
        refresh: checkStatus,
    };
})();
