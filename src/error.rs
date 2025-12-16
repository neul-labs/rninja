use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("unknown rule '{name}'")]
    UnknownRule { name: String },

    #[error("duplicate rule '{name}'")]
    DuplicateRule { name: String },

    #[error("missing required variable '{var}' in rule '{rule}'")]
    MissingVariable { var: String, rule: String },

    #[error("circular include detected: {path}")]
    CircularInclude { path: String },
}

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("unknown target '{target}'")]
    UnknownTarget { target: String },

    #[error("dependency cycle detected involving '{target}'")]
    Cycle { target: String },

    #[error("multiple rules generate '{output}'")]
    DuplicateOutput { output: String },
}

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("command failed with exit code {code}: {command}")]
    CommandFailed { command: String, code: i32 },

    #[error("subcommand failed")]
    SubcommandFailed,

    #[error("failed to spawn command: {0}")]
    SpawnError(#[from] std::io::Error),

    #[error("build stopped: {0} targets failed")]
    BuildFailed(usize),
}
