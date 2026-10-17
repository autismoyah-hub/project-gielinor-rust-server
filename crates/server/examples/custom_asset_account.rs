//! Provision only a NEW disposable experiment account, never existing users.
use gielinor_storage::{accounts::{Accounts,Save},password::PasswordHasher,sqlite::Database};
fn main()->Result<(),String>{
    let a:Vec<_>=std::env::args().collect();
    if a.len()!=5 {return Err("DATABASE ARGON2_DLL USER PASSWORD_FILE".into());}
    let mut accounts=Accounts::from_database(Database::open(std::path::Path::new(&a[1]),false).map_err(|e|e.message)?);
    accounts.initialize().map_err(|e|e.message)?;
    if accounts.load(&a[3],1).map_err(|e|e.message)?.is_some(){return Err("Refusing existing account".into());}
    let password=std::fs::read(&a[4]).map_err(|e|e.to_string())?;
    let hash=PasswordHasher::load(std::path::Path::new(&a[2])).map_err(|e|format!("{e:?}"))?.hash(&password).map_err(|e|format!("{e:?}"))?;
    let (account,character)=accounts.create(&a[3],&hash,1).map_err(|e|e.message)?;
    let data=accounts.load(&a[3],1).map_err(|e|e.message)?.ok_or("New account missing")?;
    // Existing-account login requires a completed logout record. Establish it
    // through the normal storage save contract for this newly created user only.
    accounts.save(&Save {tower_diary:data.tower_diary,tower_run:data.tower_run,tower_arrival:data.tower_arrival,
        account_id:data.account_id,character_id:data.character_id,display_name:data.display_name,
        known_device:data.known_device,modlevel:data.modlevel,coords:data.coords,
        persistent_varps_json:data.varps_json,last_login:data.last_login,run_energy:data.run_energy,
        xp_rate_hundreds:data.xp_rate_hundreds,stats:data.stats,inventories:data.inventories}).map_err(|e|e.message)?;
    if accounts.load(&a[3],1).map_err(|e|e.message)?.ok_or("New account missing")?.last_logout.is_none(){return Err("Missing initial logout marker".into());}
    println!("Created disposable account {account}, character {character}");
    Ok(())
}
