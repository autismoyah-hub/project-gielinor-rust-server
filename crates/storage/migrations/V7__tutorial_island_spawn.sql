-- New accounts begin on Tutorial Island (Learning the Ropes start house).
-- Lumbridge remains the respawn / mainland destination after tutorial completion.
UPDATE realms
SET spawn_coord = '0_26_95_22_35'
WHERE spawn_coord = '0_50_50_21_18';
