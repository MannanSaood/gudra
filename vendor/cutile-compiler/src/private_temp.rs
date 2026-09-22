// SPDX-License-Identifier: Apache-2.0
// Local modification: compiler artifacts live in a private Linux directory.
use std::{io, path::{Path, PathBuf}};

pub(crate) struct PrivateTemp(PathBuf);

impl PrivateTemp {
    pub(crate) fn new(token: &str) -> io::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::{DirBuilderExt, MetadataExt};
            // Deliberately independent of process-global TMPDIR and its ancestors.
            // Both components are root-owned, real directories. Sticky /tmp
            // prevents another UID replacing our atomically created entry.
            for path in ["/", "/tmp"] {
                let m = std::fs::symlink_metadata(path)?;
                if !m.is_dir() || m.uid() != 0
                    || (m.mode() & 0o022 != 0 && (path != "/tmp" || m.mode() & 0o1000 == 0))
                {
                    return Err(io::Error::new(io::ErrorKind::PermissionDenied, "untrusted /tmp ancestry"));
                }
            }
            if token.is_empty() || !token.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid temporary token"));
            }
            let path = Path::new("/tmp").join(format!("cutile-private-{token}"));
            std::fs::DirBuilder::new().mode(0o700).create(&path)?;
            let result = Self(path);
            let m = std::fs::symlink_metadata(&result.0)?;
            if !m.is_dir() || m.mode() & 0o777 != 0o700 {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "private directory mode"));
            }
            Ok(result)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = token;
            Err(io::Error::new(io::ErrorKind::Unsupported, "private JIT storage requires Linux"))
        }
    }

    pub(crate) fn path(&self) -> &Path { &self.0 }

    pub(crate) fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
        options.open(path)?.write_all(bytes)
    }

    pub(crate) fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let m = std::fs::symlink_metadata(path)?;
        if !m.is_file() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "compiler output is not a regular file"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if m.nlink() != 1 || m.uid() != std::fs::metadata(&self.0)?.uid() {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "compiler output owner/link mismatch"));
            }
        }
        // Only this UID and the trusted compiler can change names in this directory.
        std::fs::read(path)
    }
}

impl Drop for PrivateTemp {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn temp() -> PrivateTemp {
        PrivateTemp::new(&format!("{:x}-{:x}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))).unwrap()
    }
    #[test]
    fn exclusive_creation_and_cleanup() {
        let dir = temp();
        let path = dir.path().to_owned();
        let file = path.join("input.bc");
        dir.write(&file, b"control").unwrap();
        assert!(dir.write(&file, b"replacement").is_err());
        assert_eq!(dir.read(&file).unwrap(), b"control");
        drop(dir);
        assert!(!path.exists());
    }
    #[test]
    fn rejects_links_and_cleans_during_unwind() {
        let dir = temp();
        let path = dir.path().to_owned();
        std::os::unix::fs::symlink("/etc/passwd", path.join("output.cubin")).unwrap();
        assert!(dir.read(&path.join("output.cubin")).is_err());
        let _ = std::panic::catch_unwind(|| { let _owned = dir; panic!("test unwind"); });
        assert!(!path.exists());
    }
}
