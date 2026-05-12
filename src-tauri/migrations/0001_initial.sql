-- Initial migration. Per-domain tables live in later migrations as each slice is built.
-- This file is intentionally minimal: it just establishes the schema version row exists
-- (the _migrations table itself is created by the runner before applying any file).
PRAGMA foreign_keys = ON;
