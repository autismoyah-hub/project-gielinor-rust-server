use crate::sqlite::{Database, Error, Value};
use gielinor_world::tower_run::Run;
fn integer(value: &Value) -> Result<u32, Error> {
    match value {
        Value::Integer(value) => {
            u32::try_from(*value).map_err(|_| Error::from("Invalid Tower run integer".to_string()))
        }
        _ => Err(Error::from("Invalid Tower run value".to_string())),
    }
}
pub(crate) fn load(db: &Database, character: i64) -> Result<Run, Error> {
    let rows = db.query(
        "SELECT run_id,services,flags,clears FROM tower_run WHERE character_id=?",
        &[Value::Integer(character)],
        1,
    )?;
    let Some(row) = rows.first() else {
        return Ok(Run::default());
    };
    let flags = integer(&row[2])?;
    let services = integer(&row[1])?;
    let run = Run {
        id: integer(&row[0])?,
        services: services as u8,
        imp: flags & 1 != 0,
        victory: flags & 2 != 0,
        reward_pending: flags & 4 != 0,
        first_reward_paid: flags & 8 != 0,
        clears: integer(&row[3])?,
    };
    if flags > 15 || services > 7 || !run.valid() {
        return Err(Error::from("Invalid Tower run lifecycle".to_string()));
    }
    Ok(run)
}
pub(crate) fn save(db: &Database, character: i64, run: Run) -> Result<(), Error> {
    if !run.valid() {
        return Err(Error::from("Invalid Tower run lifecycle".to_string()));
    }
    let flags = run.imp as i64
        | ((run.victory as i64) << 1)
        | ((run.reward_pending as i64) << 2)
        | ((run.first_reward_paid as i64) << 3);
    db.execute("INSERT INTO tower_run(character_id,run_id,services,flags,clears) VALUES(?,?,?,?,?) ON CONFLICT(character_id) DO UPDATE SET run_id=excluded.run_id,services=excluded.services,flags=excluded.flags,clears=excluded.clears", &[
        Value::Integer(character), Value::Integer(run.id as i64), Value::Integer(run.services as i64), Value::Integer(flags), Value::Integer(run.clears as i64)])?;
    Ok(())
}
