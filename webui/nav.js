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
        { id: 'home', label: 'Console', href: '/webui/index.html' },
        { id: 'profile', label: 'Profile', href: '/webui/profile.html' },
        { id: 'search', label: 'Brain Search', href: '/webui/search.html' },
        { id: 'status', label: 'Status', href: '/webui/status.html' },
    ];

    // Admin/Trainer/Advanced navigation
    const ADMIN_PAGES = [
        { id: 'admin', label: 'Admin', href: '/webui/admin_dashboard.html' },
        { id: 'trainer', label: 'Trainer', href: '/webui/trainer_panel.html' },
        { id: 'logs', label: 'Logs', href: '/webui/logs.html' },
        { id: 'evolution', label: 'Evolution', href: '/webui/evolution.html' },
        { id: 'brain', label: 'Brain Graph', href: '/webui/visualize.html' },
        { id: 'logic', label: 'Logic Graph', href: '/webui/logic_visualize.html' },
        { id: 'blacklist', label: 'Blacklist', href: '/webui/admin_blacklist.html' },
        { id: 'whitelist', label: 'Whitelist', href: '/webui/admin_whitelist.html' },
        { id: 'anomalies', label: 'Anomalies', href: '/webui/admin_anomalies.html' },
        { id: 'reasoning', label: 'Reasoning', href: '/webui/admin_reasoning.html' },
    ];

    function render(activeId) {
        const container = document.getElementById('topnav');
        if (!container) return;

        const isAdmin = typeof JEEBS_ROOT_ADMIN !== 'undefined' &&
            jeebsGetToken() &&
            localStorage.getItem('jeebs_is_admin') === 'true';

        let linksHtml = PAGES.map(function (p) {
            return `<a class="topnav-link${p.id === activeId ? ' active' : ''}" href="${p.href}">${p.label}</a>`;
        }).join('');

        // Always add admin links — auth guard on each page handles access
        linksHtml += ADMIN_PAGES.map(function (p) {
            return `<a class="topnav-link${p.id === activeId ? ' active' : ''}" href="${p.href}">${p.label}</a>`;
        }).join('');

        container.className = 'topnav';
        container.innerHTML = `
            <div class="topnav-inner">
                <a class="topnav-brand" href="/webui/index.html">
                    ${LOGO_SVG}
                    <span>JeebsAI</span>
                </a>
                <button class="topnav-toggle" id="navToggle" aria-label="Toggle navigation">&#9776;</button>
                <div class="topnav-links" id="navLinks">
                    ${linksHtml}
                </div>
                <div class="topnav-status" id="navStatus">
                    <span class="dot" id="navDot"></span>
                    <span id="navStatusText">Checking...</span>
                </div>
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
            } else {
                dot.className = 'dot offline';
                text.textContent = 'Not signed in';
                localStorage.removeItem('jeebs_is_admin');
            }
        } catch (e) {
            dot.className = 'dot offline';
            text.textContent = 'Offline';
        }
    }

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
