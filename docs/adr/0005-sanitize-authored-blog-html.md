# Allow authored HTML and sanitize Blog output

Allow raw HTML in Blog Markdown to preserve authoring patterns such as
horizontally scrollable tables, then sanitize the rendered output with Ammonia
before display. Sanitization is a required part of this policy, not an optional
rendering step.

## Addendum (2025-10-24): Profile sections

The same sanitizer (`convert_markdown_to_html`) now also sanitizes
Profile Section bodies and permits `target="_blank"` on `<a>` elements, so
external links keep opening in a new tab as they did when the Profile page was
hardcoded HTML. This allowance applies to Blog bodies as well.
