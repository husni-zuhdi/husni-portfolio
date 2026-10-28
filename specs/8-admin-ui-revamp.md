# Admin UI Revamp

## Summary

The Admin area pages were rebuilt to share one visual system: navigation-only
landing, table-based lists, panel-style forms, and inline Lucide icons. All
Admin sub-pages (Profile, Blogs, Blog Tags, Talks) are covered. The HTMX
interaction model is unchanged: lists, edit forms, add forms, and delete
confirmations swap fragments from the same endpoints as before.

## What changed

### Landing page

`/admin` renders a grid of launcher cards (Profile, Blogs, Blog Tags, Talks),
each with a Lucide icon and the primary navigation links. The explicit
navigator block in the shared footer (`admin_base.html`) was removed; the
top navbar still has the same links.

### Shared components

- `templates/admin/partials/icons/`: Lucide SVGs vendored inline (no CDN, so
  they survive HTMX swaps): user, pen-line, tags, message-circle, pencil,
  trash-2, arrow-up, arrow-down, check, x, plus, eye.
- CSS component classes in `statics/input.css`:
  - `.admin_card`: landing card.
  - `.admin_panel`: form panel used by add/edit/meta forms.
  - `.admin_icon` / `.admin_icon_lg`: icon sizing. The base-layer
    `svg { fill: ... }` rule would break Lucide's stroke-based icons, these
    classes force `[&>svg]:fill-none` with higher specificity.
- Icon buttons carry `aria-label` and `title` so icon-only actions stay
  accessible.

### Tables and row partials

List pages render real `<table>` elements (thead header row plus an Actions
column). Row markup lives in a shared partial per area:

- `templates/admin/talks/partials/talk_row.html` (Date, Name, Organization, Actions)
- `templates/admin/blogs/partials/blog_row.html` (ID, Name, Tags, Actions)
- `templates/admin/blogs/tags/partials/tag_row.html` (ID, Name, Actions)
- `templates/admin/profile/partials/section_row.html` (Position, Title, Layout, Actions)

Item views (`get_*` templates) include the same partial so the fragment
returned after an accepted or canceled edit matches the list row exactly.
Regression tests lock this row/list consistency for talks and profile
sections:

- `src/handler/admin/talks/displays.rs` tests row `<tr>` versus list markup.
- `src/handler/admin/profile/displays.rs` (`test_get_section_row_matches_list_row_markup`).

Because forms cannot live directly inside `<tr>`, edit and delete flows swap
the whole row (`hx-swap="outerHTML"` on `<tr id="..._target">`); edit forms
render as a `<tr>` with a single `colspan` cell containing an `.admin_panel`
form. Blogs keeps its whole-container swap (`#blogs_target` semantics
unchanged). Add flows target the existing list containers
(`#talks_target`, `#tags_target`, `#blogs_target`, `#profile_target`) with
`innerHTML` swaps.

Action buttons use small Lucide icons: pencil (edit), trash-2 (delete),
arrow-up/arrow-down (move), check (save), x (cancel), plus (create), eye
(preview).

### Bug fix

The tags delete-confirmation template targeted the talks row
(`#talk_{{ id }}_target`) when canceling. It now targets
`#tag_{{ id }}_target` and cancels by re-rendering the tag row.

### Blog tag chips

Blog tags render as pill chips (rounded-full border, small padding) with the
singular Lucide `tag` logo beside the tag name. The styling lives in the
existing `inactive_tag`/`active_tag` component classes (no dedicated chip
class). This applies to the Admin blog table row and, because the classes are
shared, to the public blog list and blog pages where tags are also filter
buttons; the `active_tag` variant stays inverted (gray background, white
text). Hover effect is bold for both variants. Chips were grilled as a
deliberate design choice; public pages were included for consistency.

## Blog editor enhancements

The Blog add/edit forms received a separate pass after the initial revamp:

- **Tags:** replaced the native `<select multiple>` with a searchable dropdown,
  selected pill chips, and an inline "Create new tag" option.
- **Body:** split-pane markdown editor + live preview on desktop, tabbed editor
  / preview on mobile. Both debounced live preview and a manual preview button
  are available via `POST /admin/blogs/preview`.
- **Math:** MathJax was replaced with vendored Temml; authors write standard
  `$...$` / `$$...$$` delimiters.
- **Diagrams:** ` ```mermaid ` code blocks are converted to `<pre class="mermaid">`
  and rendered with vendored Pele.
- **Bugs fixed:** empty tag values no longer leak into the database, form fields
  without `=` are skipped, rendered tables are auto-wrapped in `.table-wrapper`,
  and new tags are created automatically when explicitly chosen.

## Editor script initialization and tag-less blogs

A follow-up pass fixed two interaction bugs introduced by the revamp:

- **Script init race:** the per-form inline `<script>initBlogEditor(...)</script>`
  executed before `admin-blog-editor.js` loaded inside HTMX-swapped fragments,
  so the tag widget, tab switching, and preview rendering randomly failed
  ("initBlogEditor is not defined"). The editor script now loads once from
  `admin_base.html` `<head>` and the forms are initialized globally via
  `htmx:load`; Temml/Pele preview rendering runs globally on
  `htmx:afterSwap` for `#blog_preview` through the exported
  `renderBlogPreviewContent`. Per-form inline scripts are gone.
- **Mobile preview tab:** activating the preview tab with an empty preview
  triggers the preview request automatically (tab clicks only; the eye button
  and debounced refresh do not re-trigger to avoid request loops).
- **Tag-less blogs:** blog list and lookup queries joined through
  `blog_tag_mapping` with inner joins, so blogs without any tag disappeared
  from `/blogs`, `/admin/blogs/list`, and resolved to `404` on direct lookups
  (cache briefly masked this until it drained). The Turso queries in
  `src/database/turso/blogs.rs` now use LEFT JOINs with
  `COALESCE(group_concat(...), '')`; tag-less blogs appear unfiltered and are
  hidden only while a tag filter is active. An empty `source` value (legacy
  admin-created rows) maps to `Filesystem` without an error log.
- **Tag dropdown blur race:** dropdown items were non-focusable `<div>`s;
  mousedown blurred the search input, the `focusout` handler hid the dropdown
  between mousedown and mouseup, and the click never landed — adding tags
  failed on add and edit forms while removal worked. Items now call
  `preventDefault()` on mousedown so the input keeps focus and the click
  completes.
- **Mobile preview auto-switch:** the debounced background refresh reused the
  eye button's click, which also switched the active tab — on mobile, typing
  yanked the view to the preview pane. The background refresh now runs only
  on desktop (matchMedia `min-width: 768px`) and never switches tabs; a
  `previewDirty` flag makes the preview tab re-fetch when the body changed
  since the last render.
- **Preview code-block parity:** the public blog page loads highlight.js with
  the nord theme; the Admin preview did not, so `pre`/code blocks looked
  different (affecting the surrounding mermaid frame). `admin_base.html` now
  loads the same nord.css/highlight.js assets, and the preview renderer
  runs `hljs.highlightElement()` on `pre code` inside `#blog_preview`.
- **Vendored syntax highlighting:** highlight.js 11.11.1 and the nord theme
  live in `statics/hljs/` (served via a dedicated route like Temml and Pele);
  the public blog page and the Admin area both load them locally instead of
  from unpkg.

## Known limitations

- The profile page-title section row shows `[Title]` next to its position and
  disallows deletion via the `protected` confirm template; protection logic
  lives in the operation handler, not the UI.
- Table styling uses the `.admin_table` component class (full-width,
  left-aligned) on top of the site-wide `table` base styles; per-column
  widths are left to the browser's automatic table layout.
- The shared page shell (navbar/footer outside `admin_base.html`) is still
  hardcoded and out of scope for this change.
