# Husni Portfolio

This glossary defines the domain language for the public portfolio and the people
who maintain its content.

## Language

**Portfolio**:
The public professional presentation of its owner.

**Blog**:
A written article published as part of the Portfolio.
_Avoid_: post

**Tag**:
A label that describes a Blog and helps readers find related Blogs.

**Blog tag**:
The relationship created when a Tag is assigned to a Blog.
_Avoid_: Blog-tag mapping

**Talk**:
A presentation given by the Portfolio owner at an event.

**Profile section**:
An editable block of the Profile page with a position, a heading, a markdown
body, and a Layout. The first section by position is the page title section
and cannot be deleted.
_Avoid_: profile item

**Layout**:
How a Profile section body is rendered: `stacked` (normal document flow) or
`cards` (flex-container grid). Unknown values fall back to `stacked`.

**Administrator**:
A person responsible for maintaining the Portfolio's content.
_Avoid_: Admin, when referring to a person

**Admin area**:
The restricted part of the site where the Administrator manages Blogs, Tags, and
Talks.
_Avoid_: Admin alone
