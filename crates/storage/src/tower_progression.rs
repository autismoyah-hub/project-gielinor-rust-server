use crate::sqlite::{Database, Error, Value};
use gielinor_world::tower_progression::{FloorId, Progress};

pub(crate) fn load(db: &Database, character: i64) -> Result<Progress, Error> {
    let rows = db.query("SELECT floor_id FROM tower_floor_completion WHERE character_id=? ORDER BY floor_id",
        &[Value::Integer(character)], 65535)?;
    let mut floors = Vec::with_capacity(rows.len());
    for row in rows {
        let id = match row.first() {
            Some(Value::Integer(id)) => u16::try_from(*id).ok().and_then(FloorId::new),
            _ => None,
        }.ok_or_else(|| Error::from("Invalid Tower floor identity".to_string()))?;
        floors.push(id);
    }
    let mut progress = Progress::restore(floors);
    for row in db.query("SELECT floor_id,objectives FROM tower_objective_history WHERE character_id=? ORDER BY floor_id",
        &[Value::Integer(character)], 65535)? {
        let floor = match row.first() {
            Some(Value::Integer(id)) => u16::try_from(*id).ok().and_then(FloorId::new), _ => None,
        }.ok_or_else(|| Error::from("Invalid Tower objective floor identity".to_string()))?;
        progress.restore_history(floor, mask(&row[1])?);
    }
    Ok(progress)
}
fn mask(value: &Value) -> Result<u64, Error> {
    match value {
        Value::Blob(bytes) if bytes.len() == 8 => Ok(u64::from_le_bytes(bytes.as_slice().try_into().unwrap())),
        _ => Err(Error::from("Invalid Tower objective history".to_string())),
    }
}
/// Accomplishments are monotonic. A stale snapshot cannot revoke an earned floor.
/// Called inside the existing character/inventory transaction only.
pub(crate) fn save(db: &Database, character: i64, progress: &Progress) -> Result<(), Error> {
    for floor in progress.floors() {
        db.execute("INSERT INTO tower_floor_completion(character_id,floor_id) VALUES(?,?) ON CONFLICT(character_id,floor_id) DO NOTHING",
            &[Value::Integer(character), Value::Integer(i64::from(floor.get()))])?;
    }
    for (floor, achieved) in progress.history() {
        let args = [Value::Integer(character), Value::Integer(i64::from(floor.get()))];
        let existing = db.query("SELECT objectives FROM tower_objective_history WHERE character_id=? AND floor_id=?", &args, 1)?;
        let before = existing.first().map_or(Ok(0), |row| mask(&row[0]))?;
        db.execute("INSERT INTO tower_objective_history(character_id,floor_id,objectives) VALUES(?,?,?) ON CONFLICT(character_id,floor_id) DO UPDATE SET objectives=excluded.objectives",
            &[args[0].clone(), args[1].clone(), Value::Blob((before | achieved).to_le_bytes().to_vec())])?;
    }
    Ok(())
}

/// Batched, validated one-time migration. The surrounding schema transaction
/// rolls everything back on invalid legacy data, including the version marker.
pub(crate) fn backfill(db: &Database) -> Result<(), Error> {
    let mut after = i64::MIN;
    loop {
        let rows = db.query("SELECT character_id FROM tower_run WHERE character_id>? ORDER BY character_id LIMIT 256", &[Value::Integer(after)], 256)?;
        if rows.is_empty() { return Ok(()); }
        for row in rows {
            let Value::Integer(character) = row[0] else { return Err(Error::from("Invalid Tower character identity".to_string())); };
            let run = crate::tower_run::load(db, character)?;
            let mut progress = load(db, character)?;
            progress.reconcile_cinderworks(run);
            save(db, character, &progress)?;
            after = character;
        }
    }
}
