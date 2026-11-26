-- The projection is empty at cutover; the column keeps the acquisition time under its new name.
ALTER TABLE offers RENAME COLUMN scraped_at TO acquired_at;
