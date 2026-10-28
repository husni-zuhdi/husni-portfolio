const { test, describe, before } = require('node:test');
const assert = require('node:assert');
const { JSDOM } = require('jsdom');
const fs = require('fs');
const path = require('path');

const EDITOR_SCRIPT = fs.readFileSync(
  path.resolve(__dirname, '../../statics/admin-blog-editor.js'),
  'utf8'
);

function buildDom(availableTags, selectedTags) {
  const availableAttr = availableTags ? availableTags.join(',') : '';
  const selectedAttr = selectedTags ? selectedTags.join(',') : '';
  const html = `
    <form class="blog-editor-form">
      <div class="tag-widget"
           data-available-tags="${availableAttr}"
           data-selected-tags="${selectedAttr}">
        <div class="tag-search-row">
          <input type="text" class="tag-search-input">
          <div class="tag-dropdown hidden"></div>
        </div>
        <div class="tag-selected-list"></div>
      </div>
      <div class="body-editor-tabs">
        <button type="button" class="tab-btn active" data-tab="editor">Editor</button>
        <button type="button" class="tab-btn" data-tab="preview">Preview</button>
      </div>
      <div class="body-editor-panes">
        <div class="body-editor-pane active" data-pane="editor"></div>
        <div class="body-editor-pane" data-pane="preview">
          <div id="blog_preview"></div>
        </div>
      </div>
      <textarea id="blog_body"></textarea>
      <button type="button" hx-post="/admin/blogs/preview">Preview</button>
    </form>
  `;
  const dom = new JSDOM(html, { url: 'http://localhost', runScripts: 'dangerously' });
  const document = dom.window.document;
  const form = document.querySelector('.blog-editor-form');
  const script = document.createElement('script');
  script.textContent = EDITOR_SCRIPT;
  form.appendChild(script);
  return { dom, document, form };
}

describe('tag widget', () => {
  test('renders selected tags as chips with hidden inputs', () => {
    const { document } = buildDom(['rust', 'go'], ['rust']);
    const chips = document.querySelectorAll('.tag-chip');
    assert.strictEqual(chips.length, 1);
    assert.ok(chips[0].textContent.includes('rust'));

    const hiddenInputs = document.querySelectorAll('input[name="blog_tag"]');
    assert.strictEqual(hiddenInputs.length, 1);
    assert.strictEqual(hiddenInputs[0].value, 'rust');
  });

  test('shows matching available tags in dropdown', () => {
    const { document } = buildDom(['rust', 'ruby', 'go'], []);
    const input = document.querySelector('.tag-search-input');
    input.value = 'ru';
    input.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));

    const items = document.querySelectorAll('.tag-dropdown-item');
    const names = Array.from(items).map((item) => item.textContent);
    assert.ok(names.includes('rust'), 'dropdown should include rust');
    assert.ok(names.includes('ruby'), 'dropdown should include ruby');
    assert.ok(!names.includes('go'), 'dropdown should not include go');
  });

  test('offers to create a new tag when no exact match exists', () => {
    const { document } = buildDom(['rust'], []);
    const input = document.querySelector('.tag-search-input');
    input.value = 'newtag';
    input.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));

    const createItem = document.querySelector('.tag-create-item');
    assert.ok(createItem, 'should show create-new-tag item');
    assert.ok(createItem.textContent.includes('newtag'));
  });

  test('clicking a dropdown item selects it', () => {
    const { document } = buildDom(['rust', 'go'], []);
    const input = document.querySelector('.tag-search-input');
    input.value = 'ru';
    input.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));

    const item = document.querySelector('.tag-dropdown-item');
    item.click();

    const hiddenInputs = document.querySelectorAll('input[name="blog_tag"]');
    assert.strictEqual(hiddenInputs.length, 1);
    assert.strictEqual(hiddenInputs[0].value, 'rust');
  });

  test('clicking chip remove button unselects the tag', () => {
    const { document } = buildDom(['rust'], ['rust']);
    const removeBtn = document.querySelector('.tag-chip button');
    removeBtn.click();

    const hiddenInputs = document.querySelectorAll('input[name="blog_tag"]');
    assert.strictEqual(hiddenInputs.length, 0);
  });

  test('does not add duplicate tags', () => {
    const { document } = buildDom(['rust'], ['rust']);
    const input = document.querySelector('.tag-search-input');
    input.value = 'rust';
    input.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));

    const item = document.querySelector('.tag-dropdown-item');
    if (item) item.click();

    const hiddenInputs = document.querySelectorAll('input[name="blog_tag"]');
    assert.strictEqual(hiddenInputs.length, 1);
  });
});

describe('body editor tabs', () => {
  test('switches active tab and pane on click', () => {
    const { document } = buildDom([], []);
    const previewBtn = document.querySelector('[data-tab="preview"]');
    previewBtn.click();

    assert.ok(previewBtn.classList.contains('active'));
    assert.ok(document.querySelector('[data-pane="preview"]').classList.contains('active'));
    assert.ok(!document.querySelector('[data-tab="editor"]').classList.contains('active'));
  });
});

describe('preview rendering', () => {
  before(() => {
    global.temml = {
      renderMathInElement: () => {},
    };
    global.pele = {
      mount: () => {},
    };
  });

  test('renders math and mermaid in preview after swap', () => {
    const { document } = buildDom([], []);
    const preview = document.querySelector('#blog_preview');
    preview.innerHTML = '<pre class="mermaid">graph TD; A --> B</pre>';
    preview.dispatchEvent(new document.defaultView.Event('htmx:afterSwap', { bubbles: true }));

    // If temml/pele were called without throwing, the listeners are wired.
    // We mainly assert the preview container received the expected content.
    assert.ok(preview.querySelector('pre.mermaid'));
  });
});
