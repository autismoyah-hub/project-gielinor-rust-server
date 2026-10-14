-- Unknown older diary state remains absent; do not fabricate fresh marks.
CREATE TABLE tower_diary (
    character_id INTEGER PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    tans INTEGER NOT NULL,
    offers INTEGER NOT NULL,
    bows INTEGER NOT NULL,
    shrimp_cooked INTEGER NOT NULL,
    minds_made INTEGER NOT NULL,
    arrows_made INTEGER NOT NULL,
    actions_mask INTEGER NOT NULL CHECK(actions_mask BETWEEN 0 AND 127),
    done_mask INTEGER NOT NULL CHECK(done_mask BETWEEN 0 AND 2047),
    pending_mask INTEGER NOT NULL CHECK(pending_mask BETWEEN 0 AND 3),
    bounty INTEGER NOT NULL,
    schema11 INTEGER NOT NULL CHECK(schema11 IN (0,1)),
    imp_bones INTEGER NOT NULL CHECK(imp_bones IN (0,1))
);
