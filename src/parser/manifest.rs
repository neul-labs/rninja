use std::collections::HashMap;

/// A parsed ninja manifest (build.ninja file)
#[derive(Debug, Default)]
pub struct Manifest {
    /// Top-level variables
    pub variables: HashMap<String, String>,
    /// Rule definitions
    pub rules: HashMap<String, Rule>,
    /// Build edges
    pub builds: Vec<Build>,
    /// Default targets
    pub defaults: Vec<String>,
    /// Pool definitions
    pub pools: HashMap<String, Pool>,
    /// Included files (processed in same scope)
    pub includes: Vec<String>,
    /// Subninja files (processed in separate scope)
    pub subninjas: Vec<String>,
}

/// A rule definition
#[derive(Debug, Default, Clone)]
pub struct Rule {
    pub name: String,
    /// The command to run
    pub command: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Path to depfile for implicit dependencies
    pub depfile: Option<String>,
    /// Dependency style: gcc or msvc
    pub deps: Option<String>,
    /// If true, rule is re-run if build.ninja changes
    pub generator: bool,
    /// If true, re-stat outputs after command to check if changed
    pub restat: bool,
    /// Response file path
    pub rspfile: Option<String>,
    /// Response file content
    pub rspfile_content: Option<String>,
    /// Pool to use for this rule
    pub pool: Option<String>,
    /// Additional variables
    pub variables: HashMap<String, String>,
}

/// A build edge
#[derive(Debug, Default, Clone)]
pub struct Build {
    /// Output files
    pub outputs: Vec<String>,
    /// Implicit outputs (after |)
    pub implicit_outputs: Vec<String>,
    /// Rule name
    pub rule: String,
    /// Explicit input files
    pub inputs: Vec<String>,
    /// Implicit dependencies (after |)
    pub implicit_deps: Vec<String>,
    /// Order-only dependencies (after ||)
    pub order_only_deps: Vec<String>,
    /// Build-specific variable overrides
    pub variables: HashMap<String, String>,
}

impl Build {
    /// Get all dependencies (explicit + implicit + order-only)
    pub fn all_deps(&self) -> impl Iterator<Item = &str> {
        self.inputs
            .iter()
            .chain(self.implicit_deps.iter())
            .chain(self.order_only_deps.iter())
            .map(|s| s.as_str())
    }

    /// Get all outputs (explicit + implicit)
    pub fn all_outputs(&self) -> impl Iterator<Item = &str> {
        self.outputs
            .iter()
            .chain(self.implicit_outputs.iter())
            .map(|s| s.as_str())
    }
}

/// A pool for limiting parallelism
#[derive(Debug, Clone)]
pub struct Pool {
    pub name: String,
    /// Maximum concurrent jobs
    pub depth: usize,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            name: String::new(),
            depth: 1,
        }
    }
}
