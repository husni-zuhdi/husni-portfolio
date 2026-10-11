# Use the database as the runtime source for Blogs

Store and serve Blog content from the database rather than loading it from
GitHub or the filesystem. The database is the runtime source of truth; existing
`BlogSource` values are legacy data and do not select a runtime content provider
or reliably identify a Blog's origin.
