-- Raw offers carry the instant the handler finished fetching them.
ALTER TABLE offer_details_raw RENAME COLUMN fetched_at TO acquired_at;
ALTER TABLE offer_details_raw ALTER COLUMN acquired_at DROP DEFAULT;

-- Exports have a single payload shape, so a source object has one snapshot.
ALTER TABLE offer_export
    DROP CONSTRAINT offer_export_offer_id_source_object_id_schema_version_key,
    DROP CONSTRAINT offer_export_schema_version_check,
    DROP COLUMN schema_version,
    ADD CONSTRAINT offer_export_offer_id_source_object_id_key UNIQUE (offer_id, source_object_id);
