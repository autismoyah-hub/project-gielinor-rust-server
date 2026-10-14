//! Owned SQLite representation of the Tower's character diary sidecar.
use crate::sqlite::{Database, Error, Value};
use gielinor_world::tower_diary::Progress;

fn number(value: &Value) -> Result<i32, Error> {
    match value {
        Value::Integer(n) => i32::try_from(*n)
            .map_err(|_| Error::from("Tower diary integer outside i32".to_string())),
        _ => Err(Error::from("Expected Tower diary integer".to_string())),
    }
}
fn flags<const N: usize>(value: &Value) -> Result<[bool; N], Error> {
    let mask = number(value)?;
    if !(0..(1 << N)).contains(&mask) {
        return Err(Error::from("Invalid Tower diary flags".to_string()));
    }
    Ok(std::array::from_fn(|bit| mask & (1 << bit) != 0))
}
fn mask<const N: usize>(flags: [bool; N]) -> Value {
    Value::Integer(
        flags
            .into_iter()
            .enumerate()
            .fold(0, |m, (bit, set)| m | ((set as i64) << bit)),
    )
}
pub(crate) fn load(db: &Database, character: i64) -> Result<Option<Progress>, Error> {
    let rows = db.query(
        "SELECT tans,offers,bows,shrimp_cooked,minds_made,arrows_made,actions_mask,done_mask,pending_mask,bounty,schema11,imp_bones FROM tower_diary WHERE character_id=?",
        &[Value::Integer(character)], 1,
    )?;
    let Some(row) = rows.first() else {
        return Ok(None);
    };
    let mut counts = [0; 6];
    for (index, count) in counts.iter_mut().enumerate() {
        *count = number(&row[index])?;
    }
    Ok(Some(Progress {
        counts,
        actions: flags(&row[6])?,
        done: flags(&row[7])?,
        pending: flags(&row[8])?,
        bounty: number(&row[9])?,
        schema11: flags::<1>(&row[10])?[0],
        imp_bones: flags::<1>(&row[11])?[0],
    }))
}
pub(crate) fn save(db: &Database, character: i64, progress: Progress) -> Result<(), Error> {
    let mut values = vec![Value::Integer(character)];
    values.extend(progress.counts.map(|n| Value::Integer(n as i64)));
    values.extend([
        mask(progress.actions),
        mask(progress.done),
        mask(progress.pending),
        Value::Integer(progress.bounty as i64),
        Value::Integer(progress.schema11 as i64),
        Value::Integer(progress.imp_bones as i64),
    ]);
    db.execute(
        "INSERT INTO tower_diary(character_id,tans,offers,bows,shrimp_cooked,minds_made,arrows_made,actions_mask,done_mask,pending_mask,bounty,schema11,imp_bones) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(character_id) DO UPDATE SET tans=excluded.tans,offers=excluded.offers,bows=excluded.bows,shrimp_cooked=excluded.shrimp_cooked,minds_made=excluded.minds_made,arrows_made=excluded.arrows_made,actions_mask=excluded.actions_mask,done_mask=excluded.done_mask,pending_mask=excluded.pending_mask,bounty=excluded.bounty,schema11=excluded.schema11,imp_bones=excluded.imp_bones",
        &values,
    )?;
    Ok(())
}
