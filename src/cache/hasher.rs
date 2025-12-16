use crate::error::ExecError;
use blake3::Hasher;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Compute a cache key for a build action
///
/// The key is based on:
/// - The command string
/// - Content hashes of all input files
/// - Relevant environment variables
pub fn compute_action_key(
    command: &str,
    inputs: &[&Path],
    env_vars: &[(&str, &str)],
) -> Result<String, ExecError> {
    let mut hasher = Hasher::new();

    // Hash the command
    hasher.update(b"cmd:");
    hasher.update(command.as_bytes());
    hasher.update(b"\n");

    // Hash input files (sorted for determinism)
    let mut sorted_inputs: Vec<_> = inputs.iter().collect();
    sorted_inputs.sort_by_key(|p| p.to_string_lossy().to_string());

    for input in sorted_inputs {
        if input.exists() {
            hasher.update(b"in:");
            hasher.update(input.to_string_lossy().as_bytes());
            hasher.update(b":");

            // Hash file content
            let content_hash = hash_file(input)?;
            hasher.update(content_hash.as_bytes());
            hasher.update(b"\n");
        }
    }

    // Hash environment variables (sorted for determinism)
    let mut sorted_env: Vec<_> = env_vars.iter().collect();
    sorted_env.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted_env {
        hasher.update(b"env:");
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Compute the blake3 hash of a file's contents
pub fn hash_file(path: &Path) -> Result<String, ExecError> {
    let file = File::open(path).map_err(ExecError::SpawnError)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 65536]; // 64KB buffer

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(ExecError::SpawnError)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Compute the blake3 hash of arbitrary bytes
pub fn hash_bytes(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        let hash1 = hash_file(file.path()).unwrap();
        let hash2 = hash_file(file.path()).unwrap();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // blake3 produces 256-bit (64 hex chars) hash
    }

    #[test]
    fn test_action_key_deterministic() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        let inputs = vec![file.path()];
        let env = vec![("CC", "gcc"), ("CFLAGS", "-O2")];

        let key1 = compute_action_key("gcc -c foo.c", &inputs, &env).unwrap();
        let key2 = compute_action_key("gcc -c foo.c", &inputs, &env).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_action_key_differs_by_command() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        file.flush().unwrap();

        let inputs = vec![file.path()];
        let env: Vec<(&str, &str)> = vec![];

        let key1 = compute_action_key("gcc -c foo.c", &inputs, &env).unwrap();
        let key2 = compute_action_key("gcc -c bar.c", &inputs, &env).unwrap();

        assert_ne!(key1, key2);
    }
}
