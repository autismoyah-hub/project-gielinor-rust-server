-- Preserve legacy absence: diary_progress cannot establish the bones sidecar.
ALTER TABLE tower_arrival ADD COLUMN bones_offered INTEGER
    CHECK (bones_offered IS NULL OR bones_offered IN (0,1));
