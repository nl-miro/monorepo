-- The default 1 only satisfies the restored constraint if rollback finds rows.
ALTER TABLE offer_export
    DROP CONSTRAINT offer_export_offer_id_source_object_id_key,
    ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 1
        CONSTRAINT offer_export_schema_version_check CHECK (schema_version > 0);
ALTER TABLE offer_export
    ALTER COLUMN schema_version DROP DEFAULT,
    ADD CONSTRAINT offer_export_offer_id_source_object_id_schema_version_key
        UNIQUE (offer_id, source_object_id, schema_version);

ALTER TABLE offer_details_raw ALTER COLUMN acquired_at SET DEFAULT now();
ALTER TABLE offer_details_raw RENAME COLUMN acquired_at TO fetched_at;
