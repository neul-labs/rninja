use clap::Parser;

/// rninja - a drop-in replacement for Ninja with caching
#[derive(Parser, Debug)]
#[command(name = "rninja", version, about)]
pub struct Cli {
    /// Change to DIR before doing anything else
    #[arg(short = 'C', long = "dir", value_name = "DIR")]
    pub dir: Option<std::path::PathBuf>,

    /// Specify input build file [default: build.ninja]
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub file: Option<String>,

    /// Run N jobs in parallel (0 means infinity) [default: CPU count]
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<usize>,

    /// Keep going until N jobs fail (0 means infinity) [default: 1]
    #[arg(short = 'k', long = "keep-going", value_name = "N", default_value = "1")]
    pub keep_going: usize,

    /// Do not start new jobs if the load average is greater than N
    #[arg(short = 'l', long = "load-average", value_name = "N")]
    pub load_average: Option<f64>,

    /// Dry run (don't run commands but act like they succeeded)
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Show all command lines while building
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Debugging mode (see -d list for modes)
    #[arg(short = 'd', long = "debug", value_name = "MODE")]
    pub debug: Option<String>,

    /// Run a subtool (use -t list to list subtools)
    #[arg(short = 't', long = "tool", value_name = "TOOL")]
    pub tool: Option<String>,

    /// Write a build log to FILE (experimental)
    #[arg(short = 'w', long = "log", value_name = "FILE")]
    pub log: Option<String>,

    /// Targets to build
    #[arg(trailing_var_arg = true)]
    pub targets: Vec<String>,
}

impl Cli {
    /// Check if explain mode is enabled via -d
    pub fn explain(&self) -> bool {
        self.debug.as_deref() == Some("explain")
    }

    /// Check if keepdepfile mode is enabled via -d
    pub fn keep_depfile(&self) -> bool {
        self.debug.as_deref() == Some("keepdepfile")
    }

    /// Check if stats mode is enabled via -d
    pub fn stats(&self) -> bool {
        self.debug.as_deref() == Some("stats")
    }
}
