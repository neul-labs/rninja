use crate::error::ExecError;
use std::path::PathBuf;
use std::time::SystemTime;

/// A cache entry representing a cached build result
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The command that was executed
    pub command: String,
    /// Output files and their content hashes
    pub outputs: Vec<(PathBuf, String)>,
    /// When this entry was created
    pub created: SystemTime,
}

impl CacheEntry {
    /// Serialize the entry to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, ExecError> {
        // Simple format: JSON-like but hand-rolled to avoid extra dependencies
        // Format: version|command|created_secs|created_nanos|num_outputs|path1|hash1|path2|hash2|...
        let mut data = Vec::new();

        // Version byte
        data.push(1u8);

        // Command length + command
        let cmd_bytes = self.command.as_bytes();
        data.extend_from_slice(&(cmd_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(cmd_bytes);

        // Created timestamp
        let duration = self.created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        data.extend_from_slice(&duration.as_secs().to_le_bytes());
        data.extend_from_slice(&duration.subsec_nanos().to_le_bytes());

        // Number of outputs
        data.extend_from_slice(&(self.outputs.len() as u32).to_le_bytes());

        // Each output: path_len|path|hash
        for (path, hash) in &self.outputs {
            let path_str = path.to_string_lossy();
            let path_bytes = path_str.as_bytes();
            data.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
            data.extend_from_slice(path_bytes);

            let hash_bytes = hash.as_bytes();
            data.extend_from_slice(&(hash_bytes.len() as u32).to_le_bytes());
            data.extend_from_slice(hash_bytes);
        }

        Ok(data)
    }

    /// Deserialize an entry from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, ExecError> {
        let err = || {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid cache entry",
            ))
        };

        if data.is_empty() {
            return Err(err());
        }

        let mut pos = 0;

        // Version byte
        let version = data[pos];
        pos += 1;
        if version != 1 {
            return Err(err());
        }

        // Command
        if pos + 4 > data.len() {
            return Err(err());
        }
        let cmd_len = u32::from_le_bytes(data[pos..pos + 4].try_into().map_err(|_| err())?) as usize;
        pos += 4;
        if pos + cmd_len > data.len() {
            return Err(err());
        }
        let command = String::from_utf8_lossy(&data[pos..pos + cmd_len]).to_string();
        pos += cmd_len;

        // Created timestamp
        if pos + 12 > data.len() {
            return Err(err());
        }
        let secs = u64::from_le_bytes(data[pos..pos + 8].try_into().map_err(|_| err())?);
        pos += 8;
        let nanos = u32::from_le_bytes(data[pos..pos + 4].try_into().map_err(|_| err())?);
        pos += 4;
        let created = SystemTime::UNIX_EPOCH + std::time::Duration::new(secs, nanos);

        // Number of outputs
        if pos + 4 > data.len() {
            return Err(err());
        }
        let num_outputs =
            u32::from_le_bytes(data[pos..pos + 4].try_into().map_err(|_| err())?) as usize;
        pos += 4;

        // Outputs
        let mut outputs = Vec::with_capacity(num_outputs);
        for _ in 0..num_outputs {
            // Path
            if pos + 4 > data.len() {
                return Err(err());
            }
            let path_len =
                u32::from_le_bytes(data[pos..pos + 4].try_into().map_err(|_| err())?) as usize;
            pos += 4;
            if pos + path_len > data.len() {
                return Err(err());
            }
            let path = PathBuf::from(String::from_utf8_lossy(&data[pos..pos + path_len]).to_string());
            pos += path_len;

            // Hash
            if pos + 4 > data.len() {
                return Err(err());
            }
            let hash_len =
                u32::from_le_bytes(data[pos..pos + 4].try_into().map_err(|_| err())?) as usize;
            pos += 4;
            if pos + hash_len > data.len() {
                return Err(err());
            }
            let hash = String::from_utf8_lossy(&data[pos..pos + hash_len]).to_string();
            pos += hash_len;

            outputs.push((path, hash));
        }

        Ok(Self {
            command,
            outputs,
            created,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let entry = CacheEntry {
            command: "gcc -c foo.c -o foo.o".to_string(),
            outputs: vec![
                (PathBuf::from("foo.o"), "abc123".to_string()),
                (PathBuf::from("foo.d"), "def456".to_string()),
            ],
            created: SystemTime::now(),
        };

        let data = entry.serialize().unwrap();
        let restored = CacheEntry::deserialize(&data).unwrap();

        assert_eq!(entry.command, restored.command);
        assert_eq!(entry.outputs.len(), restored.outputs.len());
        for (i, (path, hash)) in entry.outputs.iter().enumerate() {
            assert_eq!(path, &restored.outputs[i].0);
            assert_eq!(hash, &restored.outputs[i].1);
        }
    }
}
