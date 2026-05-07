//! Cache schema versioning and migrations
//!
//! Provides version tracking and migration support for the cache format.

use crate::error::ExecError;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Key used to store schema info in sled
pub const SCHEMA_KEY: &[u8] = b"__rninja_schema__";

/// Schema information stored in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    /// Schema version number
    pub version: u32,
    /// When the cache was created
    pub created_at_secs: u64,
    /// When the cache was last migrated (if ever)
    pub last_migrated_secs: Option<u64>,
    /// rninja version that created/migrated the cache
    pub rninja_version: String,
}

impl SchemaInfo {
    /// Create schema info for a new cache
    pub fn current() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: CURRENT_SCHEMA_VERSION,
            created_at_secs: now,
            last_migrated_secs: None,
            rninja_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Serialize to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, ExecError> {
        serde_json::to_vec(self).map_err(|e| {
            ExecError::SpawnError(std::io::Error::other(format!("failed to serialize schema info: {}", e),
            ))
        })
    }

    /// Deserialize from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, ExecError> {
        serde_json::from_slice(data).map_err(|e| {
            ExecError::SpawnError(std::io::Error::other(format!("failed to deserialize schema info: {}", e),
            ))
        })
    }
}

/// Migration statistics
#[derive(Debug, Default)]
pub struct MigrationStats {
    pub entries_migrated: usize,
    pub entries_failed: usize,
    pub blobs_migrated: usize,
    pub duration_ms: u64,
}

/// Check schema version and migrate if needed
pub fn check_and_migrate(db: &sled::Db, auto_migrate: bool) -> Result<SchemaInfo, ExecError> {
    match db.get(SCHEMA_KEY) {
        Ok(Some(data)) => {
            let info = SchemaInfo::deserialize(&data)?;

            if info.version < CURRENT_SCHEMA_VERSION {
                if auto_migrate {
                    tracing::info!(
                        "Migrating cache from v{} to v{}",
                        info.version,
                        CURRENT_SCHEMA_VERSION
                    );
                    migrate(db, info.version)?;

                    // Update schema info
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let new_info = SchemaInfo {
                        version: CURRENT_SCHEMA_VERSION,
                        created_at_secs: info.created_at_secs,
                        last_migrated_secs: Some(now),
                        rninja_version: env!("CARGO_PKG_VERSION").to_string(),
                    };

                    db.insert(SCHEMA_KEY, new_info.serialize()?).map_err(|e| {
                        ExecError::SpawnError(std::io::Error::other(format!("failed to update schema: {}", e),
                        ))
                    })?;

                    Ok(new_info)
                } else {
                    Err(ExecError::SpawnError(std::io::Error::other(format!(
                            "cache schema migration required: v{} -> v{}. Run with auto_migrate or use cache-migrate tool",
                            info.version, CURRENT_SCHEMA_VERSION),
                    )))
                }
            } else if info.version > CURRENT_SCHEMA_VERSION {
                Err(ExecError::SpawnError(std::io::Error::other(format!(
                        "cache was created by newer rninja (schema v{}), current version supports v{}",
                        info.version, CURRENT_SCHEMA_VERSION
                    ),
                )))
            } else {
                Ok(info)
            }
        }
        Ok(None) => {
            // New database - initialize schema
            let info = SchemaInfo::current();
            db.insert(SCHEMA_KEY, info.serialize()?).map_err(|e| {
                ExecError::SpawnError(std::io::Error::other(format!("failed to initialize schema: {}", e),
                ))
            })?;
            tracing::info!("Initialized cache schema v{}", CURRENT_SCHEMA_VERSION);
            Ok(info)
        }
        Err(e) => Err(ExecError::SpawnError(std::io::Error::other(format!("failed to read schema: {}", e),
        ))),
    }
}

/// Run migrations from one version to current
fn migrate(_db: &sled::Db, _from_version: u32) -> Result<MigrationStats, ExecError> {
    let mut stats = MigrationStats::default();
    let start = std::time::Instant::now();

    // Currently we're at v1, so no migrations needed yet
    // Future migrations would be added here like:
    //
    // if from_version < 2 {
    //     stats = migrate_v1_to_v2(db)?;
    // }
    // if from_version < 3 {
    //     let v3_stats = migrate_v2_to_v3(db)?;
    //     stats.entries_migrated += v3_stats.entries_migrated;
    // }

    stats.duration_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        "Migration complete: {} entries in {}ms",
        stats.entries_migrated,
        stats.duration_ms
    );

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_info_roundtrip() {
        let info = SchemaInfo::current();
        let data = info.serialize().unwrap();
        let restored = SchemaInfo::deserialize(&data).unwrap();

        assert_eq!(info.version, restored.version);
        assert_eq!(info.rninja_version, restored.rninja_version);
    }

    #[test]
    fn test_schema_version() {
        const { assert!(CURRENT_SCHEMA_VERSION >= 1) }
    }
}
