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
        <div class="body-editor-pane active" data-pane="editor">
          <textarea id="blog_body"></textarea>
        </div>
        <div class="body-editor-pane" data-pane="preview">
          <div id="blog_preview"></div>
        </div>
      </div>
      <button type="button" hx-post="/admin/blogs/preview" hx-target="#blog_preview">Preview</button>
    </form>
  `;
  const dom = new JSDOM(html, { url: 'http://localhost', runScripts: 'dangerously' });
  const document = dom.window.document;
  const form = document.querySelector('.blog-editor-form');
  const script = document.createElement('script');
  script.textContent = EDITOR_SCRIPT;
  form.appendChild(script);
  if (typeof dom.window.initBlogEditor === 'function') {
    dom.window.initBlogEditor(form);
  }
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

  test('dropdown item mousedown prevents default to keep input focus', () => {
    const { document } = buildDom(['rust'], []);
    const input = document.querySelector('.tag-search-input');
    input.dispatchEvent(new document.defaultView.Event('focus', { bubbles: true }));
    const item = document.querySelector('.tag-dropdown-item');
    assert.ok(item, 'dropdown should render at least one item');

    const mousedownEvent = new document.defaultView.MouseEvent('mousedown', {
      bubbles: true,
      cancelable: true,
    });
    item.dispatchEvent(mousedownEvent);
    assert.strictEqual(mousedownEvent.defaultPrevented, true);
  });

  test('create-item mousedown prevents default to keep input focus', () => {
    const { document } = buildDom(['rust'], []);
    const input = document.querySelector('.tag-search-input');
    input.value = 'kubernetes';
    input.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));
    const createItem = document.querySelector('.tag-create-item');
    assert.ok(createItem, 'create item should render for unknown tags');

    const mousedownEvent = new document.defaultView.MouseEvent('mousedown', {
      bubbles: true,
      cancelable: true,
    });
    createItem.dispatchEvent(mousedownEvent);
    assert.strictEqual(mousedownEvent.defaultPrevented, true);
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
    const previewTabBtn = document.querySelector('[data-tab="preview"]');
    previewTabBtn.click();

    assert.ok(previewTabBtn.classList.contains('active'));
    assert.ok(document.querySelector('[data-pane="preview"]').classList.contains('active'));
    assert.ok(!document.querySelector('[data-tab="editor"]').classList.contains('active'));
  });

  test('preview tab triggers preview request when preview is empty', () => {
    const { document } = buildDom([], []);
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/blogs/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });

    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 1, 'empty preview should trigger a single preview request');

    document.querySelector('.tag-search-input').dispatchEvent(
      new document.defaultView.Event('focus', { bubbles: true })
    );
    assert.strictEqual(eyeClicks, 1, 'unrelated events should not re-trigger the preview');

    eyeBtn.click();
    assert.strictEqual(eyeClicks, 2, 'eye button still requestable');
  });

  test('preview tab does not trigger duplicate requests when preview has content', () => {
    const { document } = buildDom([], []);
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/blogs/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });
    document.querySelector('#blog_preview').textContent = 'rendered';

    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 0, 'non-empty preview should not re-request');
  });

  test('preview tab re-fetches after body edits (dirty tracking)', async () => {
    const { document, dom } = buildDom([], []);
    dom.window.matchMedia = () => ({ matches: true, addListener: () => {}, removeListener: () => {} });
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/blogs/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });
    document.querySelector('#blog_preview').textContent = 'stale render';

    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 0, 'non-dirty non-empty preview is fine');

    const textarea = document.querySelector('#blog_body');
    textarea.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 50));

    document.querySelector('[data-tab="editor"]').click();
    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 1, 'dirty preview should re-request on tab switch');
  });

  test('debounced body input refreshes preview without switching tabs (desktop)', async () => {
    const { document, dom } = buildDom([], []);
    dom.window.matchMedia = () => ({ matches: true, addListener: () => {}, removeListener: () => {} });
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/blogs/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });

    const textarea = document.querySelector('#blog_body');
    textarea.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 750));

    assert.strictEqual(eyeClicks, 1, 'desktop debounce should trigger the preview request');
    assert.ok(
      document.querySelector('[data-pane="editor"]').classList.contains('active'),
      'background refresh must not switch the active pane'
    );
    assert.ok(
      !document.querySelector('[data-pane="preview"]').classList.contains('active')
    );
  });

  test('debounced body input skips preview fetch on mobile', async () => {
    const { document, dom } = buildDom([], []);
    dom.window.matchMedia = () => ({ matches: false, addListener: () => {}, removeListener: () => {} });
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/blogs/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });

    const textarea = document.querySelector('#blog_body');
    textarea.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 750));

    assert.strictEqual(eyeClicks, 0, 'mobile must not auto-fetch the preview');
  });
});

describe('preview rendering', () => {
  test('renders math and mermaid content via temml and pele', () => {
    const { document, dom } = buildDom([], []);
    const calls = { temml: 0, pele: [] };
    dom.window.temml = {
      renderMathInElement: () => {
        calls.temml += 1;
      },
    };
    dom.window.pele = {
      mount: (el, source) => {
        calls.pele.push([el.textContent, source]);
      },
    };
    const preview = document.querySelector('#blog_preview');
    preview.innerHTML = '<pre class="mermaid">graph TD; A --> B</pre>';

    if (typeof dom.window.renderBlogPreviewContent === 'function') {
      dom.window.renderBlogPreviewContent(preview);
    }
    assert.strictEqual(calls.temml, 1, 'temml.renderMathInElement should be called on the preview pane');
    assert.strictEqual(calls.pele.length, 1, 'pele.mount should be called for the mermaid block');
    assert.ok(calls.pele[0][1].includes('graph TD'), 'pele.mount should receive the mermaid source');
  });

  test('highlights code blocks when hljs is available', () => {
    const { document, dom } = buildDom([], []);
    const highlighted = [];
    dom.window.hljs = {
      highlightElement: (el) => {
        highlighted.push(el);
      },
    };
    const preview = document.querySelector('#blog_preview');
    preview.innerHTML = '<pre><code class="language-rust">fn main() {}</code></pre>';

    if (typeof dom.window.renderBlogPreviewContent === 'function') {
      dom.window.renderBlogPreviewContent(preview);
    }
    assert.strictEqual(highlighted.length, 1, 'hljs.highlightElement should run for the code block');
    assert.strictEqual(highlighted[0].tagName, 'CODE');
  });
});

function buildSectionDom() {
  const html = `
    <form class="section-editor-form">
      <div class="body-editor-tabs">
        <button type="button" class="tab-btn active" data-tab="editor">Editor</button>
        <button type="button" class="tab-btn" data-tab="preview">Preview</button>
      </div>
      <div class="body-editor-panes">
        <div class="body-editor-pane active" data-pane="editor">
          <textarea id="section_body"></textarea>
        </div>
        <div class="body-editor-pane" data-pane="preview">
          <div id="section_preview" class="blog-preview"></div>
        </div>
      </div>
      <button type="button" hx-post="/admin/profile/preview" hx-target="#section_preview">Preview</button>
    </form>
  `;
  const dom = new JSDOM(html, { url: 'http://localhost', runScripts: 'dangerously' });
  const document = dom.window.document;
  const form = document.querySelector('.section-editor-form');
  const script = document.createElement('script');
  script.textContent = EDITOR_SCRIPT;
  form.appendChild(script);
  if (typeof dom.window.initBlogEditor === 'function') {
    dom.window.initBlogEditor(form);
  }
  return { dom, document, form };
}

describe('profile section body editor (same initializer as blogs)', () => {
  test('switches active tab and pane on click', () => {
    const { document } = buildSectionDom();
    const previewTabBtn = document.querySelector('[data-tab="preview"]');
    previewTabBtn.click();

    assert.ok(previewTabBtn.classList.contains('active'));
    assert.ok(document.querySelector('[data-pane="preview"]').classList.contains('active'));
    assert.ok(!document.querySelector('[data-tab="editor"]').classList.contains('active'));
  });

  test('preview tab triggers a profile preview request when the pane is empty', () => {
    const { document } = buildSectionDom();
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/profile/preview"]');
    assert.ok(eyeBtn, 'the initializer must find the section preview button by its endpoint');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });

    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 1, 'empty preview should trigger a single preview request');
  });

  test('preview tab re-fetches after section body edits (dirty tracking)', async () => {
    const { document, dom } = buildSectionDom();
    dom.window.matchMedia = () => ({ matches: true, addListener: () => {}, removeListener: () => {} });
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/profile/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });
    document.querySelector('#section_preview').textContent = 'stale render';

    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 0, 'non-dirty non-empty preview is fine');

    const textarea = document.querySelector('#section_body');
    textarea.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 50));

    document.querySelector('[data-tab="editor"]').click();
    document.querySelector('[data-tab="preview"]').click();
    assert.strictEqual(eyeClicks, 1, 'dirty preview should re-request on tab switch');
  });

  test('debounced section body input skips preview fetch on mobile', async () => {
    const { document, dom } = buildSectionDom();
    dom.window.matchMedia = () => ({ matches: false, addListener: () => {}, removeListener: () => {} });
    let eyeClicks = 0;
    const eyeBtn = document.querySelector('[hx-post="/admin/profile/preview"]');
    eyeBtn.addEventListener('click', () => {
      eyeClicks += 1;
    });

    const textarea = document.querySelector('#section_body');
    textarea.dispatchEvent(new document.defaultView.Event('input', { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 750));

    assert.strictEqual(eyeClicks, 0, 'mobile must not auto-fetch the preview');
  });
});
