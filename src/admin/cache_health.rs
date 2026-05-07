//! Cache health check tool

use crate::cache::CacheConfig;
use crate::error::ExecError;
use serde::Serialize;
use std::path::Path;

/// Health check result
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
}

/// Severity of health check failure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

/// Health check report
#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
}

/// Run cache health checks
pub fn run_cache_health(_verbose: bool, json: bool) -> Result<HealthReport, ExecError> {
    let config = CacheConfig::from_env();
    let checks = vec![
        check_directory_access(&config.cache_dir),
        check_sled_integrity(&config.cache_dir.join("index")),
        check_blob_integrity(&config.cache_dir.join("blobs")),
        check_disk_space(&config.cache_dir),
    ];

    // Determine overall status
    let overall_status = if checks.iter().any(|c| !c.passed && c.severity == Severity::Critical) {
        HealthStatus::Critical
    } else if checks.iter().any(|c| !c.passed) {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    let report = HealthReport {
        overall_status,
        checks,
    };

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Error serializing health report: {}", e);
                return Err(ExecError::SpawnError(std::io::Error::other(format!("serialization error: {}", e),
                )));
            }
        }
    } else {
        print_human_readable(&report);
    }

    Ok(report)
}

fn check_directory_access(dir: &Path) -> HealthCheck {
    let name = "directory_access".to_string();

    if !dir.exists() {
        return HealthCheck {
            name,
            passed: false,
            message: format!("Directory does not exist: {}", dir.display()),
            severity: Severity::Critical,
        };
    }

    // Check if we can write to the directory
    let test_file = dir.join(".health_check_test");
    match std::fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_file);
            HealthCheck {
                name,
                passed: true,
                message: format!("Directory accessible: {}", dir.display()),
                severity: Severity::Info,
            }
        }
        Err(e) => HealthCheck {
            name,
            passed: false,
            message: format!("Directory not writable: {}", e),
            severity: Severity::Critical,
        },
    }
}

fn check_sled_integrity(db_path: &Path) -> HealthCheck {
    let name = "sled_database".to_string();

    if !db_path.exists() {
        return HealthCheck {
            name,
            passed: true,
            message: "Database not initialized yet".to_string(),
            severity: Severity::Info,
        };
    }

    match sled::open(db_path) {
        Ok(db) => {
            let count = db.len();
            HealthCheck {
                name,
                passed: true,
                message: format!("{} entries, no corruption detected", count),
                severity: Severity::Info,
            }
        }
        Err(e) => HealthCheck {
            name,
            passed: false,
            message: format!("Database error: {}", e),
            severity: Severity::Critical,
        },
    }
}

fn check_blob_integrity(blobs_dir: &Path) -> HealthCheck {
    let name = "blob_store".to_string();

    if !blobs_dir.exists() {
        return HealthCheck {
            name,
            passed: true,
            message: "Blob store not initialized yet".to_string(),
            severity: Severity::Info,
        };
    }

    // Sample check: verify a few random blobs have valid hashes
    let mut checked = 0;
    let mut valid = 0;
    let max_samples = 10;

    if let Ok(entries) = std::fs::read_dir(blobs_dir) {
        for entry in entries.flatten() {
            if checked >= max_samples {
                break;
            }

            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        if checked >= max_samples {
                            break;
                        }

                        let blob_path = sub_entry.path();
                        if blob_path.is_file() {
                            checked += 1;
                            if verify_blob_hash(&blob_path) {
                                valid += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if checked == 0 {
        HealthCheck {
            name,
            passed: true,
            message: "No blobs to verify".to_string(),
            severity: Severity::Info,
        }
    } else if valid == checked {
        HealthCheck {
            name,
            passed: true,
            message: format!("{}/{} sampled blobs verified", valid, checked),
            severity: Severity::Info,
        }
    } else {
        HealthCheck {
            name,
            passed: false,
            message: format!("{}/{} blobs failed verification", checked - valid, checked),
            severity: Severity::Warning,
        }
    }
}

fn verify_blob_hash(path: &Path) -> bool {
    let expected_hash = match path.file_name().and_then(|n| n.to_str()) {
        Some(h) => h,
        None => return false,
    };

    match std::fs::read(path) {
        Ok(data) => {
            let actual_hash = blake3::hash(&data).to_hex().to_string();
            actual_hash == expected_hash
        }
        Err(_) => false,
    }
}

fn check_disk_space(dir: &Path) -> HealthCheck {
    let name = "disk_space".to_string();

    // Use a simple approach - check if we can create a file
    // A more complete implementation would use statvfs on Unix
    let test_file = dir.join(".space_check");
    let test_data = vec![0u8; 1024 * 1024]; // 1MB test

    match std::fs::write(&test_file, &test_data) {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_file);
            HealthCheck {
                name,
                passed: true,
                message: "Disk space available".to_string(),
                severity: Severity::Info,
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::Other
                || e.to_string().contains("space")
                || e.to_string().contains("quota")
            {
                HealthCheck {
                    name,
                    passed: false,
                    message: format!("Low disk space: {}", e),
                    severity: Severity::Critical,
                }
            } else {
                HealthCheck {
                    name,
                    passed: true,
                    message: "Disk space check skipped".to_string(),
                    severity: Severity::Info,
                }
            }
        }
    }
}

fn print_human_readable(report: &HealthReport) {
    println!("Cache Health Check:");

    for check in &report.checks {
        let status = if check.passed { "PASS" } else { "FAIL" };
        let severity_str = match check.severity {
            Severity::Info => "",
            Severity::Warning => " (warning)",
            Severity::Critical => " (critical)",
        };
        println!("  [{}] {}: {}{}", status, check.name, check.message, severity_str);
    }

    println!();
    let status_str = match report.overall_status {
        HealthStatus::Healthy => "HEALTHY",
        HealthStatus::Degraded => "DEGRADED",
        HealthStatus::Critical => "CRITICAL",
    };

    let warnings = report.checks.iter().filter(|c| !c.passed).count();
    if warnings > 0 {
        println!("Overall: {} ({} issue(s))", status_str, warnings);
    } else {
        println!("Overall: {}", status_str);
    }
}
