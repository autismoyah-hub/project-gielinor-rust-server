//! Named per-character chemistry state; no guessed target-cache variable IDs.
use crate::sqlite::{Database,Error,Value};
use gielinor_world::chemistry::State;
pub(crate) fn load(db:&Database,character:i64)->Result<State,Error> {
    let rows=db.query("SELECT remaining,stop_on_crumble FROM chemistry_state WHERE character_id=?",&[Value::Integer(character)],1)?;
    let Some(row)=rows.first() else{return Ok(State::default());};
    let number=|v:&Value|match v {Value::Integer(n)=>i32::try_from(*n).ok(),_=>None};
    number(&row[0]).zip(number(&row[1])).and_then(|(n,s)|State::restore(n,s)).ok_or_else(||Error::from("Invalid chemistry state".to_string()))
}
pub(crate) fn save(db:&Database,character:i64,state:State)->Result<(),Error> {
    db.execute("INSERT INTO chemistry_state(character_id,remaining,stop_on_crumble) VALUES(?,?,?) ON CONFLICT(character_id) DO UPDATE SET remaining=excluded.remaining,stop_on_crumble=excluded.stop_on_crumble",&[Value::Integer(character),Value::Integer(i64::from(state.remaining())),Value::Integer(state.stop_on_crumble() as i64)])?;
    Ok(())
}
