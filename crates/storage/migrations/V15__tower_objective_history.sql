-- Historical solo accomplishments, not another active run or reward ledger.
CREATE TABLE tower_objective_history (
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    floor_id INTEGER NOT NULL CHECK(floor_id BETWEEN 1 AND 65535),
    objectives BLOB NOT NULL CHECK(typeof(objectives)='blob' AND length(objectives)=8),
    PRIMARY KEY(character_id, floor_id)
);
-- Backfill through validated Rust run decoding during initialization. This
-- preserves V13/V14 and does not invent achievements absent from saved proof.
