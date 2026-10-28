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
