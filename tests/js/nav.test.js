const { test, describe } = require('node:test');
const assert = require('node:assert');
const { JSDOM } = require('jsdom');
const fs = require('fs');
const path = require('path');

const NAV_SCRIPT = fs.readFileSync(
  path.resolve(__dirname, '../../statics/nav.js'),
  'utf8'
);

function buildDom() {
  const html = `
    <nav id="navbar_target">
      <div class="main_navbar">
        <a href="/" class="navbar_link">Home</a>
        <a href="/blogs" class="navbar_link">Blogs</a>
        <a href="/talks" class="navbar_link">Talks</a>
        <a href="/login" class="navbar_link">Login</a>
        <button id="nav_drawer_toggle_target" type="button"
                aria-controls="nav_drawer_target" aria-expanded="false">
          <svg><path d="M4 6h16"></path></svg>
        </button>
        <div id="nav_backdrop_target" class="nav_backdrop" aria-hidden="true"></div>
      </div>
      <div id="nav_drawer_target" class="nav_drawer" aria-hidden="true">
        <a href="/" class="navbar_link">Home</a>
        <a href="/blogs" class="navbar_link">Blogs</a>
        <a href="/talks" class="navbar_link">Talks</a>
      </div>
    </nav>
  `;
  const dom = new JSDOM(html, { url: 'http://localhost/', runScripts: 'dangerously' });
  const document = dom.window.document;
  const script = document.createElement('script');
  script.textContent = NAV_SCRIPT;
  document.body.appendChild(script);
  dom.window.dispatchEvent(new dom.window.Event('DOMContentLoaded'));
  return { dom, document };
}

function isBackdropOpen(document) {
  return document.getElementById('nav_backdrop_target').classList.contains('open');
}

describe('nav drawer', () => {
  test('toggle opens the drawer and syncs aria state', () => {
    const { document } = buildDom();
    const toggle = document.getElementById('nav_drawer_toggle_target');
    const drawer = document.getElementById('nav_drawer_target');

    assert.strictEqual(drawer.classList.contains('open'), false);
    toggle.click();
    assert.strictEqual(drawer.classList.contains('open'), true);
    assert.strictEqual(toggle.getAttribute('aria-expanded'), 'true');
    assert.strictEqual(drawer.getAttribute('aria-hidden'), 'false');
    assert.strictEqual(isBackdropOpen(document), true);
    assert.ok(document.body.classList.contains('overflow_hidden_nav'));
  });

  test('toggle closes the drawer again', () => {
    const { document } = buildDom();
    const toggle = document.getElementById('nav_drawer_toggle_target');
    const drawer = document.getElementById('nav_drawer_target');

    toggle.click();
    toggle.click();

    assert.strictEqual(drawer.classList.contains('open'), false);
    assert.strictEqual(toggle.getAttribute('aria-expanded'), 'false');
    assert.strictEqual(drawer.getAttribute('aria-hidden'), 'true');
    assert.strictEqual(isBackdropOpen(document), false);
    assert.strictEqual(document.body.classList.contains('overflow_hidden_nav'), false);
  });

  test('backdrop click closes the drawer', () => {
    const { document } = buildDom();
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    document.getElementById('nav_backdrop_target').click();

    assert.strictEqual(
      document.getElementById('nav_drawer_target').classList.contains('open'),
      false
    );
  });

  test('escape key closes the drawer', () => {
    const { document, dom } = buildDom();
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    document.dispatchEvent(new dom.window.KeyboardEvent('keydown', { key: 'Escape' }));

    assert.strictEqual(
      document.getElementById('nav_drawer_target').classList.contains('open'),
      false
    );
  });

  test('clicking a drawer link closes the drawer', () => {
    const { document } = buildDom();
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    const drawerLink = document.querySelector('#nav_drawer_target a');
    drawerLink.click();

    assert.strictEqual(
      document.getElementById('nav_drawer_target').classList.contains('open'),
      false
    );
  });

  test('other keys do not close the drawer', () => {
    const { document, dom } = buildDom();
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    document.dispatchEvent(new dom.window.KeyboardEvent('keydown', { key: 'ArrowUp' }));

    assert.strictEqual(
      document.getElementById('nav_drawer_target').classList.contains('open'),
      true
    );
  });
});

describe('active nav link', () => {
  test('marks the link matching the current pathname', () => {
    const { document } = buildDom();
    const links = document.querySelectorAll('#navbar_target a.navbar_link');
    const activeLinks = Array.from(links).filter((link) =>
      link.hasAttribute('aria-current')
    );

    assert.strictEqual(activeLinks.length, 2, 'exactly one link per nav level is active');
    activeLinks.forEach((link) => {
      assert.strictEqual(link.getAttribute('href'), '/', 'only Home (href=/) is active on /');
    });
  });

  test('only Home is active on the homepage when Login has data-nav-path', () => {
    const dom = new JSDOM(
      '<nav id="navbar_target"><div class="main_navbar">'
      + '<a href="/" class="navbar_link">Home</a>'
      + '<a href="/" data-nav-path="/login" class="navbar_link">Login</a>'
      + '</div></nav>',
      { url: 'http://localhost/', runScripts: 'dangerously' }
    );
    const document = dom.window.document;
    const script = document.createElement('script');
    script.textContent = NAV_SCRIPT;
    document.body.appendChild(script);
    dom.window.dispatchEvent(new dom.window.Event('DOMContentLoaded'));

    const links = document.querySelectorAll('a.navbar_link');
    assert.strictEqual(links[0].classList.contains('active'), true, 'Home active on /');
    assert.strictEqual(links[1].classList.contains('active'), false, 'Login inactive on /');
    assert.strictEqual(links[0].getAttribute('aria-current'), 'page');
    assert.strictEqual(links[1].hasAttribute('aria-current'), false);
  });

  test('login link activates on /login via data-nav-path', () => {
    const dom = new JSDOM(
      '<nav id="navbar_target"><div class="main_navbar">'
      + '<a href="/" class="navbar_link">Home</a>'
      + '<a href="/" data-nav-path="/login" class="navbar_link">Login</a>'
      + '</div></nav>',
      { url: 'http://localhost/login', runScripts: 'dangerously' }
    );
    const document = dom.window.document;
    const script = document.createElement('script');
    script.textContent = NAV_SCRIPT;
    document.body.appendChild(script);
    dom.window.dispatchEvent(new dom.window.Event('DOMContentLoaded'));

    const links = document.querySelectorAll('a.navbar_link');
    assert.strictEqual(links[0].classList.contains('active'), false, 'Home inactive on /login');
    assert.strictEqual(links[1].classList.contains('active'), true, 'Login active on /login');
  });

  test('admin parent link does not light up on child routes', () => {
    const dom = new JSDOM(
      '<nav id="navbar_target"><div class="main_navbar">'
      + '<a href="/" class="navbar_link">Home</a>'
      + '<a href="/admin" class="navbar_link">Admin</a>'
      + '<a href="/admin/blogs" class="navbar_link">Admin Blogs</a>'
      + '</div></nav>',
      { url: 'http://localhost/admin/blogs', runScripts: 'dangerously' }
    );
    const document = dom.window.document;
    const script = document.createElement('script');
    script.textContent = NAV_SCRIPT;
    document.body.appendChild(script);
    dom.window.dispatchEvent(new dom.window.Event('DOMContentLoaded'));

    const links = document.querySelectorAll('a.navbar_link');
    assert.strictEqual(links[0].classList.contains('active'), false, 'Home inactive in admin');
    assert.strictEqual(links[1].classList.contains('active'), false, 'Admin parent inactive on /admin/blogs');
    assert.strictEqual(links[2].classList.contains('active'), true, 'Admin Blogs active on /admin/blogs');
  });

  test('detail pages keep section links dark (exact matching only)', () => {
    const dom = new JSDOM(
      '<nav id="navbar_target"><div class="main_navbar"><a href="/" class="navbar_link"></a><a href="/blogs" class="navbar_link"></a></div></nav>',
      { url: 'http://localhost/blogs/12', runScripts: 'dangerously' }
    );
    const document = dom.window.document;
    const script = document.createElement('script');
    script.textContent = NAV_SCRIPT;
    document.body.appendChild(script);
    dom.window.dispatchEvent(new dom.window.Event('DOMContentLoaded'));

    const links = document.querySelectorAll('a.navbar_link');
    assert.strictEqual(links[0].classList.contains('active'), false);
    assert.strictEqual(links[1].classList.contains('active'), false);
    assert.strictEqual(links[1].hasAttribute('aria-current'), false);
  });
});

function buildAdminDom(url, hoverCapable) {
  const html = `
    <nav id="navbar_target">
      <div class="main_navbar">
        <a href="/" class="navbar_link">Home</a>
        <div class="admin_dropdown" data-admin-dropdown>
          <a href="/admin"
             class="navbar_link admin_dropdown_trigger"
             data-nav-prefix="/admin"
             aria-haspopup="menu"
             aria-expanded="false"
             aria-controls="admin_dropdown_menu_target">Admin</a>
          <div id="admin_dropdown_menu_target" role="menu" class="admin_dropdown_menu hidden">
            <a href="/admin/profile" class="navbar_link">Profile</a>
            <a href="/admin/blogs" class="navbar_link">Blogs</a>
            <a href="/admin/blogs/tags" class="navbar_link">Blog Tags</a>
            <a href="/admin/talks" class="navbar_link">Talks</a>
          </div>
        </div>
        <button id="nav_drawer_toggle_target" type="button"
                aria-controls="nav_drawer_target" aria-expanded="false">
          <svg><path d="M4 6h16"></path></svg>
        </button>
        <div id="nav_backdrop_target" class="nav_backdrop" aria-hidden="true"></div>
      </div>
      <div id="nav_drawer_target" class="nav_drawer" aria-hidden="true">
        <a href="/" class="navbar_link">Home</a>
        <div class="admin_drawer_group" data-admin-drawer-group>
          <a href="/admin"
             class="navbar_link admin_dropdown_trigger"
             aria-expanded="false"
             aria-controls="admin_drawer_group_menu_target">Admin</a>
          <div id="admin_drawer_group_menu_target" role="menu" class="admin_drawer_group_menu hidden">
            <a href="/admin" class="navbar_link">Overview</a>
            <a href="/admin/profile" class="navbar_link">Profile</a>
          </div>
        </div>
      </div>
    </nav>
  `;
  const dom = new JSDOM(html, { url: `http://localhost${url}`, runScripts: 'dangerously' });
  dom.window.matchMedia = (query) => ({ matches: hoverCapable
      ? !query.includes('hover: none')
      : query.includes('hover: none'),
    addListener: () => {},
    removeListener: () => {} });
  const document = dom.window.document;
  const script = document.createElement('script');
  script.textContent = NAV_SCRIPT;
  document.body.appendChild(script);
  dom.window.dispatchEvent(new dom.window.Event('DOMContentLoaded'));
  return { dom, document };
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describe('admin dropdown (desktop header)', () => {
  test('hover opens the menu and syncs aria state', async () => {
    const { document } = buildAdminDom('/admin', true);
    const root = document.querySelector('[data-admin-dropdown]');
    const trigger = root.querySelector('.admin_dropdown_trigger');
    const menu = document.getElementById('admin_dropdown_menu_target');

    assert.strictEqual(menu.classList.contains('hidden'), true);
    root.dispatchEvent(new document.defaultView.MouseEvent('mouseenter'));
    assert.strictEqual(menu.classList.contains('hidden'), false, 'hover must open the menu');
    assert.strictEqual(trigger.getAttribute('aria-expanded'), 'true');
    assert.strictEqual(root.classList.contains('open'), true);
  });

  test('mouse-out closes the menu after a short grace period', async () => {
    const { document } = buildAdminDom('/admin', true);
    const root = document.querySelector('[data-admin-dropdown]');
    const menu = document.getElementById('admin_dropdown_menu_target');

    root.dispatchEvent(new document.defaultView.MouseEvent('mouseenter'));
    root.dispatchEvent(new document.defaultView.MouseEvent('mouseleave'));
    assert.strictEqual(menu.classList.contains('hidden'), false, 'the grace delay keeps the menu open');
    await sleep(220);
    assert.strictEqual(menu.classList.contains('hidden'), true, 'the menu closes after the grace delay');
  });

  test('hover again during the grace delay cancels the close', async () => {
    const { document } = buildAdminDom('/admin', true);
    const root = document.querySelector('[data-admin-dropdown]');
    const menu = document.getElementById('admin_dropdown_menu_target');

    root.dispatchEvent(new document.defaultView.MouseEvent('mouseenter'));
    root.dispatchEvent(new document.defaultView.MouseEvent('mouseleave'));
    root.dispatchEvent(new document.defaultView.MouseEvent('mouseenter'));
    await sleep(220);
    assert.strictEqual(menu.classList.contains('hidden'), false, 're-entering cancels the scheduled close');
  });

  test('clicking the trigger navigates as before (no preventDefault)', () => {
    const { document } = buildAdminDom('/admin', true);
    const trigger = document.querySelector('[data-admin-dropdown] .admin_dropdown_trigger');

    const clickEvent = new document.defaultView.MouseEvent('click', {
      bubbles: true, cancelable: true,
    });
    trigger.dispatchEvent(clickEvent);
    assert.strictEqual(clickEvent.defaultPrevented, false, 'desktop click keeps navigating to /admin');
  });

  test('outside click closes the open menu', async () => {
    const { document } = buildAdminDom('/admin', true);
    const root = document.querySelector('[data-admin-dropdown]');
    const menu = document.getElementById('admin_dropdown_menu_target');

    root.dispatchEvent(new document.defaultView.MouseEvent('mouseenter'));
    assert.strictEqual(menu.classList.contains('hidden'), false);

    document.body.dispatchEvent(new document.defaultView.MouseEvent('click', { bubbles: true }));
    assert.strictEqual(menu.classList.contains('hidden'), true, 'an outside click closes the menu');
  });

  test('tapping the trigger on touch opens first, navigates on the second tap', () => {
    const { document } = buildAdminDom('/admin', false);
    const trigger = document.querySelector('[data-admin-dropdown] .admin_dropdown_trigger');
    const menu = document.getElementById('admin_dropdown_menu_target');

    const firstTap = new document.defaultView.MouseEvent('click', {
      bubbles: true, cancelable: true,
    });
    trigger.dispatchEvent(firstTap);
    assert.strictEqual(firstTap.defaultPrevented, true, 'the first touch tap opens the menu');
    assert.strictEqual(menu.classList.contains('hidden'), false);

    const secondTap = new document.defaultView.MouseEvent('click', {
      bubbles: true, cancelable: true,
    });
    trigger.dispatchEvent(secondTap);
    assert.strictEqual(secondTap.defaultPrevented, false, 'the second touch tap navigates to /admin');
  });

  test('the trigger lights on any /admin/* page via data-nav-prefix', () => {
    const { document } = buildAdminDom('/admin/blogs', true);
    const trigger = document.querySelector('[data-admin-dropdown] .admin_dropdown_trigger');
    const home = document.querySelector('#navbar_target > .main_navbar > a.navbar_link');

    assert.strictEqual(trigger.classList.contains('active'), true, 'Admin lights on /admin/blogs');
    assert.strictEqual(home.classList.contains('active'), false);
  });

  test('dropdown items keep exact-match highlighting', () => {
    const { document } = buildAdminDom('/admin/profile', true);
    const trigger = document.querySelector('[data-admin-dropdown] .admin_dropdown_trigger');
    const profileItem = document.querySelector('[data-admin-dropdown] a[href="/admin/profile"]');
    const blogsItem = document.querySelector('[data-admin-dropdown] a[href="/admin/blogs"]');

    assert.strictEqual(trigger.classList.contains('active'), true, 'the trigger lights on /admin/profile');
    assert.strictEqual(profileItem.classList.contains('active'), true, 'Profile item active');
    assert.strictEqual(blogsItem.classList.contains('active'), false, 'Blogs item dark');
  });
});

describe('admin group (mobile drawer)', () => {
  test('tapping Admin expands the group in place without navigating or closing the drawer', () => {
    const { document, dom } = buildAdminDom('/admin', false);
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    const root = document.querySelector('[data-admin-drawer-group]');
    const trigger = root.querySelector('.admin_dropdown_trigger');
    const menu = document.getElementById('admin_drawer_group_menu_target');
    const drawer = document.getElementById('nav_drawer_target');

    const tap = new document.defaultView.MouseEvent('click', {
      bubbles: true, cancelable: true,
    });
    trigger.dispatchEvent(tap);
    assert.strictEqual(tap.defaultPrevented, true, 'the first touch tap opens the group');
    assert.strictEqual(menu.classList.contains('hidden'), false, 'the group submenu expands');
    assert.strictEqual(trigger.getAttribute('aria-expanded'), 'true');
    assert.strictEqual(drawer.classList.contains('open'), true, 'the drawer stays open');
  });

  test('tapping Admin when the group is open navigates to /admin (no separate Overview item)', () => {
    const { document, dom } = buildAdminDom('/admin', false);
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    const trigger = document.querySelector('[data-admin-drawer-group] .admin_dropdown_trigger');
    trigger.click(); // opens the group

    const secondTap = new document.defaultView.MouseEvent('click', {
      bubbles: true, cancelable: true,
    });
    trigger.dispatchEvent(secondTap);
    assert.strictEqual(secondTap.defaultPrevented, false, 'the second tap navigates to /admin');
  });

  test('tapping a group child closes the drawer (navigation leaves)', () => {
    const { document, dom } = buildAdminDom('/admin/profile', false);
    const toggle = document.getElementById('nav_drawer_toggle_target');
    toggle.click();

    const trigger = document.querySelector('[data-admin-drawer-group] .admin_dropdown_trigger');
    trigger.click();

    const drawer = document.getElementById('nav_drawer_target');
    // The Profile child points at the current URL, so jsdom does not
    // attempt a real page navigation while we assert the drawer close.
    const child = document.querySelector('#admin_drawer_group_menu_target a[href="/admin/profile"]');
    child.click();
    assert.strictEqual(drawer.classList.contains('open'), false, 'child navigation closes the drawer');
  });

  test('escape closes an open dropdown along with the drawer', () => {
    const { document, dom } = buildAdminDom('/admin', true);
    const root = document.querySelector('[data-admin-dropdown]');
    root.dispatchEvent(new document.defaultView.MouseEvent('mouseenter'));

    document.dispatchEvent(
      new dom.window.KeyboardEvent('keydown', { key: 'Escape' })
    );
    assert.strictEqual(
      document.getElementById('admin_dropdown_menu_target').classList.contains('hidden'),
      true,
      'escape closes the desktop dropdown'
    );
    assert.strictEqual(
      document.getElementById('nav_drawer_target').classList.contains('open'),
      false,
      'escape still closes the drawer'
    );
  });
});
