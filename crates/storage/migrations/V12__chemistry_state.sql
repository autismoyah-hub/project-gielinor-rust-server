-- Ordinary amulet charges are per character, independent of inventory item identity.
CREATE TABLE chemistry_state (
    character_id INTEGER PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    remaining INTEGER NOT NULL CHECK(remaining BETWEEN 1 AND 5),
    stop_on_crumble INTEGER NOT NULL CHECK(stop_on_crumble IN (0,1))
);
