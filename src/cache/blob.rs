use crate::error::ExecError;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use super::hasher;

/// Content-addressed blob store for artifacts
pub struct BlobStore {
    /// Root directory for blob storage
    root: PathBuf,
}

impl BlobStore {
    /// Open or create a blob store at the given path
    pub fn open(path: &Path) -> Result<Self, ExecError> {
        fs::create_dir_all(path).map_err(ExecError::SpawnError)?;
        Ok(Self {
            root: path.to_path_buf(),
        })
    }

    /// Store a file and return its content hash
    pub fn store(&self, path: &Path) -> Result<String, ExecError> {
        // Compute hash
        let hash = hasher::hash_file(path)?;

        // Determine blob path (using first 2 chars as subdirectory for fanout)
        let blob_path = self.blob_path(&hash);

        // Skip if already exists
        if blob_path.exists() {
            return Ok(hash);
        }

        // Create parent directory
        if let Some(parent) = blob_path.parent() {
            fs::create_dir_all(parent).map_err(ExecError::SpawnError)?;
        }

        // Copy file to blob store
        let src = File::open(path).map_err(ExecError::SpawnError)?;
        let dst = File::create(&blob_path).map_err(ExecError::SpawnError)?;

        let mut reader = BufReader::new(src);
        let mut writer = BufWriter::new(dst);
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(ExecError::SpawnError)?;
            if bytes_read == 0 {
                break;
            }
            writer
                .write_all(&buffer[..bytes_read])
                .map_err(ExecError::SpawnError)?;
        }

        writer.flush().map_err(ExecError::SpawnError)?;

        // Copy file permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = path.metadata() {
                let _ = fs::set_permissions(&blob_path, meta.permissions());
            }
        }

        Ok(hash)
    }

    /// Restore a blob to a destination path
    pub fn restore(&self, hash: &str, dest: &Path) -> Result<bool, ExecError> {
        let blob_path = self.blob_path(hash);

        if !blob_path.exists() {
            return Ok(false);
        }

        // Create parent directory
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(ExecError::SpawnError)?;
        }

        // Copy blob to destination
        let src = File::open(&blob_path).map_err(ExecError::SpawnError)?;
        let dst = File::create(dest).map_err(ExecError::SpawnError)?;

        let mut reader = BufReader::new(src);
        let mut writer = BufWriter::new(dst);
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(ExecError::SpawnError)?;
            if bytes_read == 0 {
                break;
            }
            writer
                .write_all(&buffer[..bytes_read])
                .map_err(ExecError::SpawnError)?;
        }

        writer.flush().map_err(ExecError::SpawnError)?;

        // Copy permissions from blob
        #[cfg(unix)]
        {
            if let Ok(meta) = blob_path.metadata() {
                let _ = fs::set_permissions(dest, meta.permissions());
            }
        }

        Ok(true)
    }

    /// Check if a blob exists
    pub fn exists(&self, hash: &str) -> bool {
        self.blob_path(hash).exists()
    }

    /// Get the path for a blob
    fn blob_path(&self, hash: &str) -> PathBuf {
        // Use first 2 characters as subdirectory for fanout
        if hash.len() >= 2 {
            self.root.join(&hash[..2]).join(hash)
        } else {
            self.root.join(hash)
        }
    }

    /// Run garbage collection - remove unreferenced blobs
    pub fn gc(&self) -> Result<BlobGcStats, ExecError> {
        let mut stats = BlobGcStats::default();

        // For now, just report stats without actually removing anything
        // In a full implementation, we'd track references and remove orphans

        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(subentries) = fs::read_dir(entry.path()) {
                        for subentry in subentries.flatten() {
                            if let Ok(meta) = subentry.metadata() {
                                stats.total_blobs += 1;
                                stats.total_bytes += meta.len();
                            }
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Get total size of all blobs
    pub fn total_size(&self) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(subentries) = fs::read_dir(entry.path()) {
                        for subentry in subentries.flatten() {
                            if let Ok(meta) = subentry.metadata() {
                                total += meta.len();
                            }
                        }
                    }
                }
            }
        }
        total
    }
}

/// Blob GC statistics
#[derive(Debug, Default)]
pub struct BlobGcStats {
    pub total_blobs: usize,
    pub total_bytes: u64,
    pub blobs_removed: usize,
    pub bytes_freed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_store_and_restore() {
        let dir = tempdir().unwrap();
        let store = BlobStore::open(dir.path()).unwrap();

        // Create a test file
        let mut src = NamedTempFile::new().unwrap();
        src.write_all(b"hello world").unwrap();
        src.flush().unwrap();

        // Store it
        let hash = store.store(src.path()).unwrap();
        assert!(store.exists(&hash));

        // Restore to new location
        let dest = dir.path().join("restored.txt");
        assert!(store.restore(&hash, &dest).unwrap());
        assert!(dest.exists());

        // Verify content
        let content = fs::read_to_string(&dest).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_deduplication() {
        let dir = tempdir().unwrap();
        let store = BlobStore::open(dir.path()).unwrap();

        // Create two files with same content
        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(b"same content").unwrap();
        file1.flush().unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(b"same content").unwrap();
        file2.flush().unwrap();

        // Store both
        let hash1 = store.store(file1.path()).unwrap();
        let hash2 = store.store(file2.path()).unwrap();

        // Should have same hash (deduplication)
        assert_eq!(hash1, hash2);
    }
}
