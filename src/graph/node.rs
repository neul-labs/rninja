/// A node in the build graph
#[derive(Debug, Default, Clone)]
pub struct Node {
    /// The output path this node produces
    pub path: String,
    /// The expanded command to run
    pub command: Option<String>,
    /// Pre-computed hash of the command (for fast comparison in up-to-date checks)
    pub command_hash: u64,
    /// Human-readable description
    pub description: Option<String>,
    /// Dependencies (paths of other nodes)
    pub deps: Vec<String>,
    /// Rule name
    pub rule: String,
    /// Is this a phony target?
    pub is_phony: bool,
    /// Is this a source file (no build rule)?
    pub is_source: bool,
    /// Should we re-stat after running to check if output changed?
    pub restat: bool,
    /// Path to depfile for discovered dependencies
    pub depfile: String,
    /// Pool name for parallelism limiting
    pub pool: Option<String>,
    /// Is this a generator rule?
    pub generator: bool,
    /// Response file path
    pub rspfile: Option<String>,
    /// Response file content
    pub rspfile_content: Option<String>,
}

impl Node {
    /// Check if this node needs to be rebuilt
    pub fn needs_rebuild(&self) -> bool {
        if self.is_phony {
            return true; // phony targets always "run"
        }

        if self.is_source {
            return false; // source files don't need building
        }

        // Check if output exists
        let output_path = std::path::Path::new(&self.path);
        if !output_path.exists() {
            return true;
        }

        // Check if any dependency is newer
        let output_mtime = match output_path.metadata().and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => return true,
        };

        for dep in &self.deps {
            let dep_path = std::path::Path::new(dep);
            if let Ok(meta) = dep_path.metadata() {
                if let Ok(dep_mtime) = meta.modified() {
                    if dep_mtime > output_mtime {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get a display string for this node (description or command)
    pub fn display(&self) -> &str {
        self.description
            .as_deref()
            .or(self.command.as_deref())
            .unwrap_or(&self.path)
    }
}
