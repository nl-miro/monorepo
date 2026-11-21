CREATE INDEX command_entries_correlation_idx ON command_entries ((meta->>'correlation_id'));
CREATE INDEX event_entries_correlation_idx ON event_entries ((meta->>'correlation_id'));
