-- Permanent solo conquests, separate from V13's current Cinderworks attempt.
-- No fabricated reward, timestamp, credit balance or group policy.
CREATE TABLE tower_floor_completion (
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    floor_id INTEGER NOT NULL CHECK(floor_id BETWEEN 1 AND 65535),
    PRIMARY KEY(character_id, floor_id)
);
INSERT INTO tower_floor_completion(character_id, floor_id)
    SELECT character_id, 1 FROM tower_run WHERE clears > 0;
