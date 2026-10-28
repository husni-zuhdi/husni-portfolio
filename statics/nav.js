/// Navigation drawer helpers for the mobile header layout.

function updateNavDrawer(open) {
    var drawer = document.getElementById('nav_drawer_target');
    var backdrop = document.getElementById('nav_backdrop_target');
    var toggle = document.getElementById('nav_drawer_toggle_target');

    if (!drawer || !toggle) return;

    drawer.classList.toggle('open', open);
    drawer.setAttribute('aria-hidden', open ? 'false' : 'true');
    if (backdrop) {
        backdrop.classList.toggle('open', open);
        backdrop.setAttribute('aria-hidden', open ? 'false' : 'true');
    }
    toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    document.body.classList.toggle('overflow_hidden_nav', open);
}

function closeNavDrawer() {
    updateNavDrawer(false);
}

function onNavDrawerToggleClick() {
    var drawer = document.getElementById('nav_drawer_target');
    if (!drawer) return;
    updateNavDrawer(!drawer.classList.contains('open'));
}

function onNavDrawerKeydown(event) {
    if (event.key === 'Escape') closeNavDrawer();
}

/// Highlight the navbar link matching the current pathname.
///
/// A link's target route comes from `data-nav-path` when set (for links whose
/// href is a generic fallback, like Login), otherwise from its href.
/// A route is active only when the pathname equals it exactly (ignoring a
/// trailing slash), so parent links like `/admin` stay dark on child pages
/// like `/admin/blogs`, and detail pages like `/blogs/12` show no highlight.
function navLinkMatchesRoute(link) {
    var target = link.dataset.navPath || link.getAttribute('href');
    if (!target) return false;
    var pathname = window.location.pathname.replace(/\/+$/, '') || '/';
    var normalizedTarget = target.replace(/\/+$/, '') || '/';
    return pathname === normalizedTarget;
}

function syncNavActiveLink() {
    var links = document.querySelectorAll('a.navbar_link[href], a.navbar_link[data-nav-path]');
    links.forEach(function (link) {
        var active = navLinkMatchesRoute(link);
        link.classList.toggle('active', active);
        if (active) {
            link.setAttribute('aria-current', 'page');
        } else {
            link.removeAttribute('aria-current');
        }
    });
}

function initNavDrawer() {
    var drawer = document.getElementById('nav_drawer_target');
    var toggle = document.getElementById('nav_drawer_toggle_target');
    var backdrop = document.getElementById('nav_backdrop_target');

    if (drawer && toggle) {
        drawer.querySelectorAll('a, form button').forEach(function (el) {
            el.removeEventListener('click', closeNavDrawer);
            el.addEventListener('click', closeNavDrawer);
        });

        if (toggle) {
            toggle.removeEventListener('click', onNavDrawerToggleClick);
            toggle.addEventListener('click', onNavDrawerToggleClick);
        }

        if (backdrop) {
            backdrop.removeEventListener('click', closeNavDrawer);
            backdrop.addEventListener('click', closeNavDrawer);
        }

        document.removeEventListener('keydown', onNavDrawerKeydown);
        document.addEventListener('keydown', onNavDrawerKeydown);
    }

    syncNavActiveLink();
}

// Start event listeners
window.addEventListener('DOMContentLoaded', initNavDrawer);
window.addEventListener('htmx:afterSettle', initNavDrawer);
document.addEventListener('htmx:pushedIntoHistory', syncNavActiveLink);
