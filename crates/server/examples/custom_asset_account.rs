//! Provision only a NEW disposable experiment account, never existing users.
use gielinor_storage::{accounts::Accounts,password::PasswordHasher,sqlite::Database};
fn main()->Result<(),String>{
    let a:Vec<_>=std::env::args().collect();
    if a.len()!=5 {return Err("DATABASE ARGON2_DLL USER PASSWORD_FILE".into());}
    let mut accounts=Accounts::from_database(Database::open(std::path::Path::new(&a[1]),false).map_err(|e|e.message)?);
    accounts.initialize().map_err(|e|e.message)?;
    if accounts.load(&a[3],1).map_err(|e|e.message)?.is_some(){return Err("Refusing existing account".into());}
    let password=std::fs::read(&a[4]).map_err(|e|e.to_string())?;
    let hash=PasswordHasher::load(std::path::Path::new(&a[2])).map_err(|e|format!("{e:?}"))?.hash(&password).map_err(|e|format!("{e:?}"))?;
    let (account,character)=accounts.create(&a[3],&hash,1).map_err(|e|e.message)?;
    println!("Created disposable account {account}, character {character}");
    Ok(())
}
