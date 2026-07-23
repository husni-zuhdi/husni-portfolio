# Frontend Architecture

**Version:** 0.3.5  
**Source:** `templates/**/*.html`, `src/model/templates.rs`, `src/model/templates_admin.rs`, `statics/`, `tailwind.config.js`

---

## Goal

Server-rendered HTML using Askama templates with HTMX for partial page updates. TailwindCSS for styling with dark mode support.

---

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Templating | Askama | 0.15.0 |
| Interactivity | HTMX | 2.0.6 (CDN) |
| Styling | TailwindCSS | via CLI |
| Syntax Highlighting | highlight.js | 11.11.1 (CDN) |
| Math Rendering | MathJax | 4.0 (CDN) |
| Dark Mode | Custom JS | `/theme.js` |

---

## Template Hierarchy

```
base.html                        (public base layout)
├── profile.html                 (home page)
├── blogs.html                   (blog listing)
├── blog.html                    (single blog post)
├── talks.html                   (talk listing)
├── version.html                 (build info)
├── statuses/
│   ├── 401_unauthorized.html
│   ├── 404_not_found.html
│   ├── 418_i_am_a_teapot.html
│   └── 500_internal_server_error.html
├── auth/
│   ├── login.html               (partial - no extends)
│   ├── login_retry.html         (partial)
│   ├── login_success.html       (partial)
│   └── logout.html              (extends base.html)
└── admin/
    ├── admin_base.html          (admin base layout - standalone)
    ├── admin.html               (extends admin_base)
    ├── talks/
    │   ├── talks.html           (extends admin_base)
    │   ├── list_talks.html      (partial)
    │   ├── get_talk.html        (partial)
    │   ├── get_add_talk.html    (partial)
    │   ├── get_edit_talk.html   (partial)
    │   └── get_delete_talk.html (partial)
    ├── blogs/
    │   ├── blogs.html           (extends admin_base)
    │   ├── list_blogs.html      (partial)
    │   ├── get_blog.html        (partial)
    │   ├── get_add_blog.html    (partial)
    │   ├── get_edit_blog.html   (partial)
    │   ├── get_delete_blog.html (partial)
    │   └── tags/
    │       ├── tags.html        (extends admin_base)
    │       ├── list_tags.html   (partial)
    │       ├── get_tag.html     (partial)
    │       ├── get_add_tag.html (partial)
    │       ├── get_edit_tag.html(partial)
    │       └── get_delete_tag.html (partial)
```

**Total: 34 HTML templates** (12 full-page, 22 partials)

---

## Base Layouts

### `base.html` (Public)

| Section | Content |
|---------|---------|
| `<head>` | Meta charset, description, author, viewport. HTMX 2.0.6 CDN. Favicon. `/styles.css`. Dark mode script. |
| Navbar | Logo, nav links (Home, Blogs, Talks), dark/light toggle, Login (hx-get="/login") |
| Main | `<div id="main_body_target">` with `{% block content %}` |
| Footer | Social links (email, GitHub, LinkedIn). Navigation. Heraclitus quote. |

### `admin_base.html` (Admin)

Standalone HTML document (does NOT extend `base.html`) with the same structure:

| Section | Content |
|---------|---------|
| Navbar | Admin, Admin Blogs, Admin Talks links. Dark toggle. Logout (hx-delete="/logout"). |
| Main | `{% block content %}` in `#main_body_target` |
| Footer | Social links + Admin Navigator column (Admin, Admin Blogs, Admin Blog Tags, Admin Talks) |

---

## Dark Mode

**Source:** `statics/theme.js`, `tailwind.config.js`

### Implementation

- TailwindCSS `darkMode: 'class'` — toggles `.dark` class on `<html>`.
- Theme persistence via `localStorage` key `color-theme`.
- On page load: check `localStorage` → check `prefers-color-scheme` media query → apply.
- Toggle button in navbar swaps sun/moon SVG icons.

### Color Scheme

Custom color defined in `tailwind.config.js`:
- `nord`: `#2E3440` (Nord Polar Night)

---

## Styling

### TailwindCSS Pipeline

```
statics/input.css → tailwindcss CLI → statics/styles.css
```

**Source:** `Taskfile.yml:86-88`

### Build Commands

| Task | Command |
|------|---------|
| Dev build | `tailwindcss -i ./statics/input.css -o ./statics/styles.css` |
| Production | Same command (run during `docker-build`) |

---

## HTMX Integration

### Public Pages

| Page | HTMX Usage |
|------|------------|
| Navbar Login | `hx-get="/login"` → swaps into `#main_body_target` |
| Blog tags | `hx-get="/blogs?tags={tag}"` → full page navigation |

### Admin Pages

All admin CRUD uses HTMX for in-place updates:

| Action | HTMX Method | Target | Response |
|--------|-------------|--------|----------|
| Load list | `hx-get="/admin/{entity}/list"` | `#{entity}_target` | Partial HTML |
| Open add form | `hx-get="/admin/{entity}/add"` | `#{entity}_target` | Form partial |
| Submit add | `hx-post="/admin/{entity}/add"` | `#{entity}_target` | Re-rendered list |
| Open edit form | `hx-get="/admin/{entity}/{id}/edit"` | `#blogs_target` or element | Form partial |
| Submit edit | `hx-put="/admin/{entity}/{id}/edit"` | `#blogs_target` | Re-rendered entity |
| Open delete confirm | `hx-get="/admin/{entity}/{id}/delete"` | element | Confirmation partial |
| Submit delete | `hx-delete="/admin/{entity}/{id}/delete"` | `#{entity}_target` | Re-rendered list |
| Tag search | `hx-get="/admin/blogs/tags/search"` | `#tags_target` | Filtered list |
| Login form | `hx-post="/login"` | `#login_button_notif_target` | Success/retry notification |
| Logout | `hx-delete="/logout"` | (redirect via HX-Redirect) | Logout page |

### HTMX Trigger Patterns

| Pattern | Usage |
|---------|-------|
| Default (click) | Buttons, links |
| `input changed delay:500ms` | Tag search input (debounced) |
| `hx-swap="outerHTML"` | Navigating between admin sections (blogs ↔ tags) |

---

## Template Details

### Public Pages

#### `profile.html`

The portfolio home page. Static content with:

| Section | Content |
|---------|---------|
| Overview | "SRE by job, Software Engineer by passion" |
| Work | Accelbyte (2022-Now), Tokopedia (2022-2022) |
| Volunteers | DevOps Jogja, Bangkit Academy |
| Skills | Cloud, K8s, Programming, Terraform, Linux |
| Projects | Portfolio, Sapa.AI, Bachelor Thesis |
| Interests | Running, Sci-fi, Books, Climbing |

#### `blog.html`

Single blog post view. Loads external libraries:
- `highlight.js` (Nord theme, v11.11.1) for code blocks
- MathJax v4 for math rendering (with linebreak overflow config)
- Renders `{{body|safe}}` (pre-converted markdown→HTML)

#### `blogs.html`

Blog listing with tag filtering:
- Iterates `blogs` vector
- Each blog shows name (linked to `/blogs/{id}`) and tags
- Tags rendered as buttons: `active_tag` class for selected, `inactive_tag` for unselected
- Empty tags skipped via HTML comments

#### `talks.html`

Talk listing:
- Shows date, name (linked to `media_link` if present), org (linked to `org_link`)

---

### Admin Pages

#### Common Patterns

All admin entity pages follow this structure:

```
admin_{entity}.html (extends admin_base)
  └── Load list via hx-get → #{entity}_target
       └── list_{entity}.html (partial)
            └── Each item: Edit button (*) + Delete button (x)
            └── Edit/Delete swap in-place
```

#### Blog Add/Edit Forms

Two-column layout:
- **Left (1/3):** ID (readonly), name, tags (multi-select `<select>`), submit/cancel
- **Right (2/3):** Markdown textarea (10 rows x 60 cols)

Textarea notes (rendered in the form):
- Inline math: use double backslash (`\\\\`)
- Tables: wrap in `<div style="overflow-x:auto">` for responsive display

#### Talk Add/Edit Forms

Single-column form:
- ID (readonly), name, date (HTML5 date picker), media link, org name, org link

---

## Static Assets

### Directory Structure

```
statics/
├── favicon/          (favicon files, served at /statics/*)
├── input.css         (TailwindCSS input)
├── styles.css        (compiled TailwindCSS output)
└── theme.js          (dark mode script)
```

---

## Status Pages

| Code | Template | Content |
|------|----------|---------|
| 401 | `401_unauthorized.html` | Red "401", "Unauthorized", home link |
| 404 | `404_not_found.html` | "Page not found", home link |
| 418 | `418_i_am_a_teapot.html` | "I am a teapot", braille art ASCII teapot |
| 500 | `500_internal_server_error.html` | "Internal Server Error", try again later |

---

## External Dependencies (CDN)

| Library | Version | Purpose | Loaded on |
|---------|---------|---------|-----------|
| HTMX | 2.0.6 | Partial page updates | All pages |
| highlight.js | 11.11.1 | Code syntax highlighting | `blog.html` |
| MathJax | 4.0 | LaTeX math rendering | `blog.html` |
