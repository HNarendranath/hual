use std::io;
use std::path::{Path, PathBuf};

#[cfg(windows)]
fn hide(path: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, SetFileAttributesW};

    // Path -> OsStr -> windows u16 encoded -> null terminate -> Vec<u16>
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let ok = unsafe { SetFileAttributesW(wide.as_ptr(), FILE_ATTRIBUTE_HIDDEN) };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(windows))]
fn hide(_path: &Path) -> io::Result<()> {
    Ok(())
}

pub fn ensure(root: &Path) -> io::Result<PathBuf> {
    let dir = root.join(".hual");
    std::fs::create_dir_all(&dir)?;
    hide(&dir)?;
    Ok(dir)
}
