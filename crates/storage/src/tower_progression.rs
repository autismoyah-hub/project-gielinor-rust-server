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
    Ok(Progress::restore(floors))
}
/// Accomplishments are monotonic. A stale snapshot cannot revoke an earned floor.
/// Called inside the existing character/inventory transaction only.
pub(crate) fn save(db: &Database, character: i64, progress: &Progress) -> Result<(), Error> {
    for floor in progress.floors() {
        db.execute("INSERT INTO tower_floor_completion(character_id,floor_id) VALUES(?,?) ON CONFLICT(character_id,floor_id) DO NOTHING",
            &[Value::Integer(character), Value::Integer(i64::from(floor.get()))])?;
    }
    Ok(())
}
