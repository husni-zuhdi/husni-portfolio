# Use libsql for SQLite and Turso

Use libsql for both local SQLite and remote Turso so the two supported modes
share one database adapter and query path. SQLx was considered for SQLite but
rejected because a second adapter would add a low-leverage seam; the historical
native-linking conflict between SQLx and Turso reinforced this choice. Evaluate
any future data source separately.
