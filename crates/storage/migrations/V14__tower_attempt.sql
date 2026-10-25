-- Tower framework attempts (#119 slice B). V14 is reserved for B's Tower storage.
-- `record` is the canonical 52-byte attempt record (world tower_run::attempt); `state` and
-- `last_transition` are checked copies for the one-live index. No room or progress state
-- is stored: a stored attempt can't resume a run.
CREATE TABLE tower_attempts (
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    attempt_id INTEGER NOT NULL CHECK(attempt_id >= 1),
    record BLOB NOT NULL CHECK(length(record) = 52),
    state INTEGER NOT NULL CHECK(state BETWEEN 0 AND 6),
    last_transition INTEGER NOT NULL CHECK(last_transition >= 1),
    PRIMARY KEY(character_id, attempt_id)
);
-- At most one live (Active 0 or Completed 1) attempt per character.
CREATE UNIQUE INDEX tower_attempts_one_live ON tower_attempts(character_id) WHERE state IN (0, 1);
-- One row per committed durable transition: the exact canonical request, for retries.
CREATE TABLE tower_attempt_transitions (
    character_id INTEGER NOT NULL,
    attempt_id INTEGER NOT NULL,
    transition_id INTEGER NOT NULL CHECK(transition_id >= 1),
    request BLOB NOT NULL,
    PRIMARY KEY(character_id, attempt_id, transition_id),
    FOREIGN KEY(character_id, attempt_id) REFERENCES tower_attempts(character_id, attempt_id) ON DELETE CASCADE
);
