use crate::native::Library;
use std::{
    ffi::{c_char, c_void, CStr, CString},
    path::Path,
    ptr,
};
type Handle = *mut c_void;
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
    Real(f64),
}
#[derive(Debug, Clone)]
pub struct Error {
    pub code: i32,
    pub message: String,
}
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self { code: -1, message }
    }
}
struct Api {
    open: unsafe extern "C" fn(*const c_char, *mut Handle, i32, *const c_char) -> i32,
    close: unsafe extern "C" fn(Handle) -> i32,
    errmsg: unsafe extern "C" fn(Handle) -> *const c_char,
    prepare:
        unsafe extern "C" fn(Handle, *const c_char, i32, *mut Handle, *mut *const c_char) -> i32,
    finalize: unsafe extern "C" fn(Handle) -> i32,
    step: unsafe extern "C" fn(Handle) -> i32,
    bind_count: unsafe extern "C" fn(Handle) -> i32,
    bind_int: unsafe extern "C" fn(Handle, i32, i64) -> i32,
    bind_double: unsafe extern "C" fn(Handle, i32, f64) -> i32,
    bind_null: unsafe extern "C" fn(Handle, i32) -> i32,
    bind_text: unsafe extern "C" fn(Handle, i32, *const c_char, i32, isize) -> i32,
    bind_blob: unsafe extern "C" fn(Handle, i32, *const c_void, i32, isize) -> i32,
    columns: unsafe extern "C" fn(Handle) -> i32,
    column_type: unsafe extern "C" fn(Handle, i32) -> i32,
    column_int: unsafe extern "C" fn(Handle, i32) -> i64,
    column_double: unsafe extern "C" fn(Handle, i32) -> f64,
    column_text: unsafe extern "C" fn(Handle, i32) -> *const u8,
    column_blob: unsafe extern "C" fn(Handle, i32) -> *const u8,
    column_bytes: unsafe extern "C" fn(Handle, i32) -> i32,
    last_id: unsafe extern "C" fn(Handle) -> i64,
    busy_timeout: unsafe extern "C" fn(Handle, i32) -> i32,
    _library: Library,
}
impl Api {
    fn load() -> Result<Self, Error> {
        let lib = Library::system("winsqlite3.dll")?;
        macro_rules! sym {
            ($name:literal) => {
                unsafe { std::mem::transmute(lib.symbol($name)?) }
            };
        }
        Ok(Self {
            open: sym!("sqlite3_open_v2"),
            close: sym!("sqlite3_close_v2"),
            errmsg: sym!("sqlite3_errmsg"),
            prepare: sym!("sqlite3_prepare_v2"),
            finalize: sym!("sqlite3_finalize"),
            step: sym!("sqlite3_step"),
            bind_count: sym!("sqlite3_bind_parameter_count"),
            bind_int: sym!("sqlite3_bind_int64"),
            bind_double: sym!("sqlite3_bind_double"),
            bind_null: sym!("sqlite3_bind_null"),
            bind_text: sym!("sqlite3_bind_text"),
            bind_blob: sym!("sqlite3_bind_blob"),
            columns: sym!("sqlite3_column_count"),
            column_type: sym!("sqlite3_column_type"),
            column_int: sym!("sqlite3_column_int64"),
            column_double: sym!("sqlite3_column_double"),
            column_text: sym!("sqlite3_column_text"),
            column_blob: sym!("sqlite3_column_blob"),
            column_bytes: sym!("sqlite3_column_bytes"),
            last_id: sym!("sqlite3_last_insert_rowid"),
            busy_timeout: sym!("sqlite3_busy_timeout"),
            _library: lib,
        })
    }
}
pub struct Database {
    handle: Handle,
    api: Api,
}
struct Statement<'a> {
    handle: Handle,
    db: &'a Database,
}
impl Drop for Statement<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.db.api.finalize)(self.handle);
        }
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            (self.api.close)(self.handle);
        }
    }
}
impl Database {
    pub fn memory() -> Result<Self, Error> {
        Self::open_name(":memory:", false)
    }
    pub fn open(path: &Path, read_only: bool) -> Result<Self, Error> {
        let name = path
            .to_str()
            .ok_or_else(|| Error::from("Database path is not UTF-8".to_string()))?;
        Self::open_name(name, read_only)
    }
    fn open_name(name: &str, read_only: bool) -> Result<Self, Error> {
        let api = Api::load()?;
        let name = CString::new(name).map_err(|e| Error::from(e.to_string()))?;
        let mut handle = ptr::null_mut();
        let result = unsafe {
            (api.open)(
                name.as_ptr(),
                &mut handle,
                if read_only { 1 } else { 2 | 4 },
                ptr::null(),
            )
        };
        if handle.is_null() {
            return Err(Error {
                code: result,
                message: "SQLite allocation failed".into(),
            });
        }
        let db = Self { handle, api };
        db.check(result)?;
        db.check(unsafe { (db.api.busy_timeout)(handle, 1000) })?;
        db.execute("PRAGMA foreign_keys=ON", &[])?;
        Ok(db)
    }
    fn check(&self, code: i32) -> Result<(), Error> {
        if code == 0 {
            Ok(())
        } else {
            Err(Error {
                code,
                message: unsafe { CStr::from_ptr((self.api.errmsg)(self.handle)) }
                    .to_string_lossy()
                    .into_owned(),
            })
        }
    }
    fn statement(&self, sql: &str, args: &[Value]) -> Result<Statement<'_>, Error> {
        let sql = CString::new(sql).map_err(|e| Error::from(e.to_string()))?;
        let mut handle = ptr::null_mut();
        let mut tail = ptr::null();
        let code =
            unsafe { (self.api.prepare)(self.handle, sql.as_ptr(), -1, &mut handle, &mut tail) };
        if code != 0 {
            if !handle.is_null() {
                unsafe {
                    (self.api.finalize)(handle);
                }
            }
            self.check(code)?;
        }
        if handle.is_null() {
            return Err(Error::from("Empty SQL statement".to_string()));
        }
        let statement = Statement { handle, db: self };
        if !tail.is_null()
            && !unsafe { CStr::from_ptr(tail) }
                .to_bytes()
                .iter()
                .all(u8::is_ascii_whitespace)
        {
            return Err(Error::from(
                "One SQL statement per call is required".to_string(),
            ));
        }
        if unsafe { (self.api.bind_count)(handle) } as usize != args.len() {
            return Err(Error::from("SQL parameter count mismatch".to_string()));
        }
        for (index, value) in args.iter().enumerate() {
            let index = index as i32 + 1;
            let code = unsafe {
                match value {
                    Value::Null => (self.api.bind_null)(handle, index),
                    Value::Integer(v) => (self.api.bind_int)(handle, index, *v),
                    Value::Real(v) => (self.api.bind_double)(handle, index, *v),
                    Value::Text(v) => {
                        let len = i32::try_from(v.len())
                            .map_err(|_| Error::from("Text too long".to_string()))?;
                        (self.api.bind_text)(handle, index, v.as_ptr().cast(), len, -1)
                    }
                    Value::Blob(v) => {
                        let len = i32::try_from(v.len())
                            .map_err(|_| Error::from("Blob too long".to_string()))?;
                        (self.api.bind_blob)(handle, index, v.as_ptr().cast(), len, -1)
                    }
                }
            };
            self.check(code)?;
        }
        Ok(statement)
    }
    pub fn execute(&self, sql: &str, args: &[Value]) -> Result<(), Error> {
        let stmt = self.statement(sql, args)?;
        let code = unsafe { (self.api.step)(stmt.handle) };
        if code == 101 {
            Ok(())
        } else {
            self.check(code)
        }
    }
    pub fn query(&self, sql: &str, args: &[Value], limit: usize) -> Result<Vec<Vec<Value>>, Error> {
        let stmt = self.statement(sql, args)?;
        let mut rows = Vec::new();
        loop {
            let code = unsafe { (self.api.step)(stmt.handle) };
            if code == 101 {
                break;
            }
            if code != 100 {
                self.check(code)?;
            }
            if rows.len() >= limit {
                return Err(Error::from("Query row limit exceeded".to_string()));
            }
            let mut row = Vec::new();
            for col in 0..unsafe { (self.api.columns)(stmt.handle) } {
                let value = unsafe {
                    match (self.api.column_type)(stmt.handle, col) {
                        1 => Value::Integer((self.api.column_int)(stmt.handle, col)),
                        2 => Value::Real((self.api.column_double)(stmt.handle, col)),
                        3 | 4 => {
                            let text = (self.api.column_type)(stmt.handle, col) == 3;
                            let ptr = if text {
                                (self.api.column_text)(stmt.handle, col)
                            } else {
                                (self.api.column_blob)(stmt.handle, col)
                            };
                            let len = (self.api.column_bytes)(stmt.handle, col) as usize;
                            let bytes = if len == 0 {
                                &[][..]
                            } else if ptr.is_null() {
                                return Err(Error::from(
                                    "SQLite returned null column data".to_string(),
                                ));
                            } else {
                                std::slice::from_raw_parts(ptr, len)
                            };
                            if text {
                                Value::Text(
                                    String::from_utf8(bytes.to_vec())
                                        .map_err(|e| Error::from(e.to_string()))?,
                                )
                            } else {
                                Value::Blob(bytes.to_vec())
                            }
                        }
                        _ => Value::Null,
                    }
                };
                row.push(value);
            }
            rows.push(row);
        }
        Ok(rows)
    }
    pub fn last_insert_id(&self) -> i64 {
        unsafe { (self.api.last_id)(self.handle) }
    }
    /// Trusted migration text only. Parameters must use `execute`/`query` instead.
    pub fn migration_script(&self, sql: &str) -> Result<(), Error> {
        let text = CString::new(sql).map_err(|e| Error::from(e.to_string()))?;
        let mut cursor = text.as_ptr();
        while unsafe { *cursor } != 0 {
            let mut handle = ptr::null_mut();
            let mut tail = ptr::null();
            let code =
                unsafe { (self.api.prepare)(self.handle, cursor, -1, &mut handle, &mut tail) };
            if code != 0 {
                if !handle.is_null() {
                    unsafe {
                        (self.api.finalize)(handle);
                    }
                }
                self.check(code)?;
            }
            if tail.is_null() || tail == cursor {
                return Err(Error::from("Migration parser did not advance".to_string()));
            }
            cursor = tail;
            if handle.is_null() {
                continue;
            }
            let stmt = Statement { handle, db: self };
            let code = unsafe { (self.api.step)(stmt.handle) };
            if code != 101 {
                self.check(code)?;
            }
        }
        Ok(())
    }
    pub fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.execute("BEGIN IMMEDIATE", &[])?;
        struct Rollback<'a>(&'a Database, bool);
        impl Drop for Rollback<'_> {
            fn drop(&mut self) {
                if self.1 {
                    let _ = self.0.execute("ROLLBACK", &[]);
                }
            }
        }
        let mut rollback = Rollback(self, true);
        let value = operation(self)?;
        self.execute("COMMIT", &[])?;
        rollback.1 = false;
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binds_data_and_rolls_back_partial_writes() {
        let mut db = Database::memory().unwrap();
        db.execute(
            "CREATE TABLE accounts(id INTEGER PRIMARY KEY, name TEXT UNIQUE, data BLOB)",
            &[],
        )
        .unwrap();
        let name = "O'Brien; DROP TABLE accounts;";
        db.execute(
            "INSERT INTO accounts VALUES(?, ?, ?)",
            &[
                Value::Integer(1),
                Value::Text(name.into()),
                Value::Blob(vec![0, 1, 255]),
            ],
        )
        .unwrap();
        let fail = db.transaction(|tx| {
            tx.execute("INSERT INTO accounts VALUES(2,'other',NULL)", &[])?;
            tx.execute(
                "INSERT INTO accounts VALUES(3,?,NULL)",
                &[Value::Text(name.into())],
            )
        });
        assert!(fail.is_err());
        assert_eq!(
            db.query("SELECT id,name,data FROM accounts", &[], 10)
                .unwrap(),
            vec![vec![
                Value::Integer(1),
                Value::Text(name.into()),
                Value::Blob(vec![0, 1, 255])
            ]]
        );
    }
}
