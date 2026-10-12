use std::{
    ffi::{c_void, CString},
    os::windows::ffi::OsStrExt,
    path::Path,
};
#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}
pub struct Library(*mut c_void);
impl Library {
    pub fn system(name: &str) -> Result<Self, String> {
        Self::load(Path::new(name), 0x800)
    }
    pub fn absolute(path: &Path) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("DLL path must be absolute".into());
        }
        Self::load(path, 0x1100)
    }
    fn load(path: &Path, flags: u32) -> Result<Self, String> {
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        if wide.contains(&0) {
            return Err("NUL in DLL path".into());
        }
        wide.push(0);
        let handle = unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), flags) };
        if handle.is_null() {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            Ok(Self(handle))
        }
    }
    /// Caller must use the exact exported function signature and keep this library alive.
    pub unsafe fn symbol(&self, name: &str) -> Result<*mut c_void, String> {
        let name = CString::new(name).map_err(|e| e.to_string())?;
        let ptr = GetProcAddress(self.0, name.as_ptr().cast());
        if ptr.is_null() {
            Err(format!("Missing native symbol {}", name.to_string_lossy()))
        } else {
            Ok(ptr)
        }
    }
}
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0);
        }
    }
}
