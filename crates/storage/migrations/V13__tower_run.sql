-- Direction B state is separate from the legacy eleven-row diary.
-- Absence means the character has not begun the Cinderworks route.
CREATE TABLE tower_run (
    character_id INTEGER PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    run_id INTEGER NOT NULL CHECK(run_id BETWEEN 0 AND 4294967295),
    services INTEGER NOT NULL CHECK(services BETWEEN 0 AND 7),
    flags INTEGER NOT NULL CHECK(flags BETWEEN 0 AND 15),
    clears INTEGER NOT NULL CHECK(clears BETWEEN 0 AND 4294967295)
);
