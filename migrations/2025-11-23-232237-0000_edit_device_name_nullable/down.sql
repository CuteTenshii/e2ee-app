-- This file should undo anything in `up.sql`
ALTER TABLE devices
    ALTER COLUMN name DROP NOT NULL;
