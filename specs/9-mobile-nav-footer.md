# Mobile Navigation and Footer

## Summary

The public and Admin base templates share one mobile-first header and footer
redesign. On small widths the previously stacked 220px-tall navbar became a
compact sticky single-row header with a slide-down navigation drawer; the
footer dropped its duplicate navigation block. Desktop layouts are preserved.

## Decisions (grilled and agreed)

- Navbar is sticky (`sticky top-0 z-40`) at all breakpoints, mobile and desktop.
- Navbar markup is duplicated in `base.html` and `admin_base.html` rather than
  extracted into a partial — matching the existing convention where footer
  markup is duplicated verbatim.
- Mobile side padding tightened `px-8` → `px-4` on navbar and footer only;
  `.main_body` content padding is intentionally untouched (separate pass).
- The theme toggle stays visible in the mobile header row (logo · theme ·
  hamburger); the Login link and admin Logout form live inside the drawer.
- Drawer interactions: closes on any link click (including HTMX Login/Logout),
  Escape, and backdrop click; body scroll is locked while open; the a11y
  contract is `aria-expanded`/`aria-controls` on the toggle (a disclosure, not
  a modal); opening animates with a 180ms slide-down; no focus trap.
- Footer: the quote block is untouched; the Navigator link list is hidden
  below `lg:` (it duplicates the header); grid gaps tighten to `gap-4` on
  mobile.
- The dead `footer { bottom-0 }` base-layer rule was removed (no `position`
  was ever set, so it did nothing).

## Implementation notes

- `statics/nav.js` owns the drawer: open/close state, `aria-expanded` /
  `aria-hidden` sync, body scroll lock (`overflow_hidden_nav` class), Escape /
  backdrop / link-click close, and pathname-based active-link highlighting
  (`syncNavActiveLink` sets `.active` and `aria-current="page"`). It rebinds
  on `htmx:afterSettle` and resyncs active links on `htmx:pushedIntoHistory`
  (the Login link swaps content client-side and replaces the URL).
- Active-link matching is exact: a route matches only when the pathname
  equals it (trailing slashes ignored), so parent links (`/admin`) stay dark
  on child pages (`/admin/blogs`) and detail pages (`/blogs/12`) show no
  highlight. Login uses `data-nav-path="/login"` because its `href` is `/`
  as a no-JS fallback (`GET /login` returns a bare fragment); the matcher
  prefers `data-nav-path` over `href`.
- Active-link styling lives on `.navbar_link.active` (same look as hover).
- The Admin area navbar and drawer include a `Home` link (`href="/"`) so
  admin users can return to the public homepage without logging out.
- The `partials/theme_button.html` partial was refactored from element IDs to
  classes (`.theme_button`, `.theme_icon_dark`, `.theme_icon_light`) because
  the partial is now included twice per page (desktop row + mobile row);
  `statics/theme.js` binds every button and syncs all icon pairs together.
- `.nav_drawer`/`.nav_backdrop` use a component-layer open-state class (`.open`)
  toggled by JS instead of Tailwind's `hidden` utility — the utilities layer
  would otherwise override component-layer display changes (same cascade
  issue as `.tag-dropdown.hidden`).
- `nav_drawer_slide_down` keyframes live at the bottom of `statics/input.css`.
- The `/statics/nav.js` route mirrors `/statics/theme.js` (`ServeFile`).

## Admin dropdown (consolidated Admin links)

The Admin navbar consolidates the three "Admin *" links into one "Admin"
dropdown (Profile, Blogs, Blog Tags, Talks management, where Blog Tags was
previously reachable only from the `/admin` landing cards):

- **Desktop header**: the trigger stays a real link to `/admin`. Hover or
  keyboard focus opens the menu with a 150ms mouse-out grace delay (Escape,
  outside click/tap, and navigation away also close it); a desktop click on
  "Admin" still navigates to the landing page. The menu is an absolute
  panel (`.admin_dropdown_menu`) under the trigger with a rotating chevron.
- **Touch (md+ without hover)**: `matchMedia('(hover: hover)')` decides —
  the first tap opens the menu, a second tap on the trigger navigates to
  `/admin`. No user-agent sniffing.
- **Mobile drawer**: `Admin` becomes an in-place expandable group
  (`.admin_drawer_group`): on touch, the first tap expands the indented
  submenu (Profile, Blogs, Blog Tags, Talks) and the second tap on the
  trigger navigates to `/admin` — the same tap-then-navigate contract as
  the header trigger, so there is no separate Overview item; child taps
  navigate and close the drawer. Collapse behavior and its trade-off are
  covered in "Trade-off: the drawer Admin group has no collapse gesture"
  below.
- **Highlighting**: the trigger carries `data-nav-prefix="/admin"` and
  lights on any `/admin/*` page (wayfinding while the child links are
  inside a menu); dropdown/drawer items keep exact-match highlighting, and
  `Home`/public links are unchanged.
- **Accessibility**: `aria-haspopup="menu"` + JS-synced `aria-expanded` on
  the trigger, `role="menu"` panels of plain link items, chevron rotation
  as the open cue.
- `statics/nav.js` owns all dropdown and drawer-group behavior alongside
  the drawer (re-init on the same `htmx:afterSettle` cycle; the group menu
  open state is not persisted across loads); covered by
  `tests/js/nav.test.js` (hover/grace/outside/Escape, tap-then-navigate,
  prefix highlighting, drawer group toggle and child navigation).

## Known limitations

- The hardcoded `aria-current="page"` was removed from the initial markup;
  until `nav.js` runs there is no active-link indication (pure server-side
  templates cannot know the pathname without per-page context).
- With exact matching, routes without a navbar link of their own (blog and
  talk detail pages) show no active highlight — intentional per the
  "only highlight active pages" behavior.
- The drawer re-binds link-close listeners on every `htmx:afterSettle`; the
  remove-before-add pattern keeps this idempotent, but binding cost scales
  with drawer link count (single digits today).

## Trade-off: the drawer Admin group has no collapse gesture

**Status: accepted (user decision, 2026-10-28).** The mobile drawer "Admin"
group shares one tap contract with the desktop header trigger:
**first tap expands the submenu, second tap navigates to `/admin`**. A
deliberate consequence — once the group is expanded, tapping the trigger
again cannot collapse it, because that gesture must navigate (this is what
made the separate "Overview" item removable). The expand state resets when
the drawer closes (backdrop, Escape, child navigation) or the page changes;
the group menu open state is never persisted across loads.

If in-page collapse is ever needed, options are a dedicated third gesture,
a chevron-prop toggle (currently the chevron is inside the `<a>`), or
restoring a collapsed-state tap at the cost of a two-tap route to `/admin`
— none implemented.
