-- Rust-owned projection of Kotlin FloorOneScript's arrival sidecar state.
-- Kept with character saves so inventory clearing and its once-only marker commit together.
CREATE TABLE tower_arrival (
    character_id INTEGER PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    kit_given INTEGER NOT NULL DEFAULT 0 CHECK (kit_given IN (0,1)),
    staff_taken INTEGER NOT NULL DEFAULT 0 CHECK (staff_taken IN (0,1)),
    diary_progress INTEGER NOT NULL DEFAULT 0 CHECK (diary_progress IN (0,1))
);
