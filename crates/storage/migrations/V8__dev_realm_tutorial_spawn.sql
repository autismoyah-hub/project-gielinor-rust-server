-- V7 intended to move every Lumbridge spawn to Tutorial Island, but the local
-- `dev` realm (server.toml default) can still be on Lumbridge after manual
-- realm-config edits or older DBs. Ensure all such realms spawn on the island.
UPDATE realms
SET spawn_coord = '0_26_95_22_35'
WHERE spawn_coord = '0_50_50_21_18';
