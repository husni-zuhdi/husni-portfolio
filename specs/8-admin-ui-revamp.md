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

## Known limitations

- The profile page-title section row shows `[Title]` next to its position and
  disallows deletion via the `protected` confirm template; protection logic
  lives in the operation handler, not the UI.
- Table styling uses the `.admin_table` component class (full-width,
  left-aligned) on top of the site-wide `table` base styles; per-column
  widths are left to the browser's automatic table layout.
- The shared page shell (navbar/footer outside `admin_base.html`) is still
  hardcoded and out of scope for this change.
