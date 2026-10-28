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
    if (event.key === 'Escape') {
        closeNavDrawer();
        closeAdminDropdowns();
    }
}

// Admin dropdown helpers: the desktop "Admin" navbar link opens a menu of
// the management areas, and the mobile drawer "Admin" link toggles the same
// areas in place. The trigger keeps href="/admin" so desktop clicks still
// navigate to the landing page.

function isHoverCapable() {
    return !window.matchMedia || window.matchMedia('(hover: hover)').matches;
}

function findAdminDropdowns() {
    return document.querySelectorAll('[data-admin-dropdown]');
}

function findDrawerAdminGroup() {
    return document.querySelector('[data-admin-drawer-group]');
}

function updateAdminDropdown(root, open) {
    if (!root) return;
    var trigger = root.querySelector('.admin_dropdown_trigger');
    var menu = root.querySelector('.admin_dropdown_menu, .admin_drawer_group_menu');
    if (!trigger || !menu) return;

    root.classList.toggle('open', open);
    menu.classList.toggle('hidden', !open);
    trigger.setAttribute('aria-expanded', open ? 'true' : 'false');
}

function closeAdminDropdowns() {
    findAdminDropdowns().forEach(function (root) {
        updateAdminDropdown(root, false);
    });
}

function initAdminDropdown(root, hoverGraceTimer) {
    if (!root || root.dataset.dropdownInitialized) return;
    root.dataset.dropdownInitialized = 'true';

    var trigger = root.querySelector('.admin_dropdown_trigger');
    if (!trigger) return;
    var isHeaderDropdown = root.hasAttribute('data-admin-dropdown');
    var isDrawerGroup = root.hasAttribute('data-admin-drawer-group');

    if (isHoverCapable() && isHeaderDropdown) {
        // Desktop: hover opens with a short grace delay so diagonal cursor
        // paths do not flicker the menu closed.
        root.addEventListener('mouseenter', function () {
            clearTimeout(hoverGraceTimer);
            updateAdminDropdown(root, true);
        });
        root.addEventListener('mouseleave', function () {
            clearTimeout(hoverGraceTimer);
            hoverGraceTimer = setTimeout(function () {
                updateAdminDropdown(root, false);
            }, 150);
        });
        root.addEventListener('focusin', function () {
            updateAdminDropdown(root, true);
        });
    }

    // Both triggers share one tap contract: a hover-capable desktop keeps
    // the default navigation (the drawer is hidden there anyway); a touch
    // device opens the menu on the first tap and navigates on the second.
    trigger.addEventListener('click', function (event) {
        if (isHoverCapable()) {
            return;
        }
        if (!root.classList.contains('open')) {
            event.preventDefault();
            updateAdminDropdown(root, true);
        }
    });

    if (isHeaderDropdown) {
        root.addEventListener('click', function (event) {
            var openRoots = findAdminDropdowns();
            openRoots.forEach(function (other) {
                if (other !== root && !other.contains(event.target)) {
                    updateAdminDropdown(other, false);
                }
            });
        });
    }
}

var adminOutsideCloseBound = false;

function initAdminDropdowns() {
    if (!adminOutsideCloseBound) {
        adminOutsideCloseBound = true;
        // One durable document listener: tap or click outside any open
        // dropdown closes it. Bound once because `document` survives HTMX
        // document-level swaps that re-run the inits.
        document.addEventListener('click', function (event) {
            findAdminDropdowns().forEach(function (root) {
                if (!root.contains(event.target)) {
                    updateAdminDropdown(root, false);
                }
            });
        });
    }

    var hoverGraceTimer = null;
    findAdminDropdowns().forEach(function (root) {
        initAdminDropdown(root, hoverGraceTimer);
    });
    var drawerRoot = findDrawerAdminGroup();
    if (drawerRoot) {
        initAdminDropdown(drawerRoot, hoverGraceTimer);
    }
}

/// Highlight the navbar link matching the current pathname.
///
/// A link's target route comes from `data-nav-path` when set (for links whose
/// href is a generic fallback, like Login), otherwise from its href. A link
/// carrying `data-nav-prefix` is active when the pathname starts with that
/// prefix (the dropdown trigger keeps highlighting on any /admin/* page).
/// Every other route is active only when the pathname equals it exactly
/// (ignoring a trailing slash), so parent links like `/admin` stay dark on
/// child pages like `/admin/blogs`, and detail pages like `/blogs/12` show
/// no highlight.
function navLinkMatchesRoute(link) {
    var target = link.dataset.navPath || link.getAttribute('href');
    if (!target) return false;
    var pathname = window.location.pathname.replace(/\/+$/, '') || '/';
    var normalizedTarget = target.replace(/\/+$/, '') || '/';
    if (link.dataset.navPrefix) {
        var normalizedPrefix = link.dataset.navPrefix.replace(/\/+$/, '');
        return pathname === normalizedPrefix
            || pathname.startsWith(normalizedPrefix + '/');
    }
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
            // The drawer Admin group trigger expands the group on the first
            // tap and navigates on the second; closing the drawer on its
            // click would fight the expand contract.
            if (el.closest('[data-admin-drawer-group]')
                && el.classList.contains('admin_dropdown_trigger')) {
                return;
            }
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

    initAdminDropdowns();
    syncNavActiveLink();
}

// Start event listeners
window.addEventListener('DOMContentLoaded', initNavDrawer);
window.addEventListener('htmx:afterSettle', initNavDrawer);
document.addEventListener('htmx:pushedIntoHistory', syncNavActiveLink);
