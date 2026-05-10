// Many modules contain code used by some binary targets but not all.
// The main rninja binary compiles all modules directly; suppressing
// dead-code warnings keeps CI practical while the project matures.
#![allow(dead_code)]
#![allow(unused_imports)]

mod admin;
mod buildlog;
mod cache;
mod cli;
mod config;
mod error;
mod executor;
mod graph;
mod metrics;
mod output;
mod parser;
mod server;
mod trace;

use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::Cli;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Handle tool mode (-t)
    if let Some(tool) = &cli.tool {
        return run_tool(tool, &cli);
    }

    // Normal build mode
    run_build(&cli)
}

fn run_build(cli: &Cli) -> Result<()> {
    // Change to build directory if specified
    let build_dir = if let Some(dir) = &cli.dir {
        std::env::set_current_dir(dir)?;
        std::path::Path::new(dir)
    } else {
        std::path::Path::new(".")
    };

    // Parse the build file
    let build_file = cli.file.as_deref().unwrap_or("build.ninja");
    let build_path = std::path::Path::new(build_file);
    let manifest = parser::parse_file(build_path)?;

    // Build the dependency graph
    let graph = graph::Graph::from_manifest(&manifest)?;

    // Determine targets to build
    let targets: Vec<&str> = if cli.targets.is_empty() {
        manifest.defaults.iter().map(|s| s.as_str()).collect()
    } else {
        cli.targets.iter().map(|s| s.as_str()).collect()
    };

    if targets.is_empty() {
        anyhow::bail!("no targets specified and no default target");
    }

    // Fast path: check if everything is up-to-date without creating executor
    let log = buildlog::BuildLog::open(build_dir);
    let mut mtime_cache = buildlog::MtimeCache::new();

    if buildlog::quick_uptodate_check(&graph, &targets, &log, &mut mtime_cache) {
        if cli.json {
            output::JsonEvent::NoWorkToDo.emit();
        } else {
            println!("ninja: no work to do.");
        }
        return Ok(());
    }

    // Determine output mode
    let output_mode = if cli.json {
        output::OutputMode::Json
    } else {
        output::OutputMode::Human
    };

    // Create executor and run
    let executor = executor::Executor::new(executor::Config {
        parallelism: cli.jobs.unwrap_or_else(num_cpus::get),
        dry_run: cli.dry_run,
        verbose: cli.verbose,
        keep_going: cli.keep_going,
        explain: cli.explain(),
        stats: cli.stats(),
        cache_config: cache::CacheConfig::from_env(),
        output_mode,
        trace_file: cli.trace.as_ref().map(std::path::PathBuf::from),
        runtime: None,
    });

    executor.run(&graph, &targets)?;

    Ok(())
}

fn run_tool(tool: &str, cli: &Cli) -> Result<()> {
    match tool {
        "list" => {
            println!("rninja subtools:");
            println!("    cache-gc     run cache garbage collection");
            println!("    cache-health check cache health and integrity");
            println!("    cache-stats  show cache statistics");
            println!("    clean        remove built files");
            println!("    cleandead    clean built files no longer produced by manifest");
            println!("    commands     list all commands required to rebuild given targets");
            println!("    compdb       dump JSON compilation database to stdout");
            println!("    config       show config file locations and generate sample config");
            println!("    deps         show dependencies stored in the deps log");
            println!("    graph        output graphviz dot file for targets");
            println!("    inputs       list all inputs required to rebuild given targets");
            println!("    path         find dependency path between two targets");
            println!("    query        show inputs/outputs for a path");
            println!("    recompact    recompact ninja-internal data structures");
            println!("    restat       restat all outputs in the build log");
            println!("    rules        list all rules");
            println!("    targets      list targets by their rule or depth in the DAG");
        }
        "cache-stats" => {
            use crate::admin::run_cache_stats;
            let json = cli.json;
            run_cache_stats(cli.verbose, json)?;
        }
        "cache-gc" => {
            use crate::admin::{run_cache_gc, GcOptions};
            let max_age_days = cli.targets.first().and_then(|s| s.parse().ok());
            let options = GcOptions {
                dry_run: cli.dry_run,
                max_age_days,
                remove_orphans: true,
                ..Default::default()
            };
            run_cache_gc(options, cli.verbose)?;
        }
        "cache-health" => {
            use crate::admin::run_cache_health;
            let json = cli.json;
            run_cache_health(cli.verbose, json)?;
        }
        "clean" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let mut removed = 0;
            for output in graph.outputs() {
                let path = std::path::Path::new(output);
                if path.exists() {
                    if cli.verbose {
                        println!("Remove {}", output);
                    }
                    if !cli.dry_run {
                        std::fs::remove_file(path)?;
                    }
                    removed += 1;
                }
            }

            // Also clean .ninja_log and .ninja_deps if they exist
            for log_file in &[".ninja_log", ".ninja_deps"] {
                let path = std::path::Path::new(log_file);
                if path.exists() && !cli.dry_run {
                    std::fs::remove_file(path)?;
                    removed += 1;
                }
            }

            println!("Removed {} files", removed);
        }
        "targets" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;

            for build in &manifest.builds {
                for output in &build.outputs {
                    println!("{}: {}", output, build.rule);
                }
            }
        }
        "commands" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let targets: Vec<&str> = if cli.targets.is_empty() {
                manifest.defaults.iter().map(|s| s.as_str()).collect()
            } else {
                cli.targets.iter().map(|s| s.as_str()).collect()
            };

            for target in targets {
                if let Some(node) = graph.get_node(target) {
                    if let Some(cmd) = &node.command {
                        println!("{}", cmd);
                    }
                }
            }
        }
        "graph" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let targets: Vec<&str> = if cli.targets.is_empty() {
                manifest.defaults.iter().map(|s| s.as_str()).collect()
            } else {
                cli.targets.iter().map(|s| s.as_str()).collect()
            };

            println!("{}", graph.to_dot(&targets));
        }
        "query" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            for target in &cli.targets {
                println!("{}:", target);
                if let Some(inputs) = graph.inputs_for(target) {
                    println!("  input:");
                    for input in inputs {
                        println!("    {}", input);
                    }
                }
                let outputs = graph.outputs_for(target);
                if !outputs.is_empty() {
                    println!("  outputs:");
                    for output in outputs {
                        println!("    {}", output);
                    }
                }
            }
        }
        "path" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            if cli.targets.len() != 2 {
                anyhow::bail!("path tool requires exactly two targets");
            }

            let from = &cli.targets[0];
            let to = &cli.targets[1];

            match graph.find_path(from, to) {
                Some(path) => {
                    for (i, node) in path.iter().enumerate() {
                        if i > 0 {
                            println!("  -> {}", node);
                        } else {
                            println!("{}", node);
                        }
                    }
                }
                None => {
                    println!("No path from {} to {}", from, to);
                }
            }
        }
        "deps" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let targets: Vec<&str> = if cli.targets.is_empty() {
                graph.outputs()
            } else {
                cli.targets.iter().map(|s| s.as_str()).collect()
            };

            for target in targets {
                if let Some(inputs) = graph.inputs_for(target) {
                    if !inputs.is_empty() {
                        println!("{}:", target);
                        for input in inputs {
                            println!("    {}", input);
                        }
                    }
                }
            }
        }
        "inputs" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let targets: Vec<&str> = if cli.targets.is_empty() {
                manifest.defaults.iter().map(|s| s.as_str()).collect()
            } else {
                cli.targets.iter().map(|s| s.as_str()).collect()
            };

            // Collect all transitive inputs
            let mut all_inputs = std::collections::HashSet::new();
            let order = graph.topological_order(&targets)?;
            for target in order {
                if let Some(node) = graph.get_node(&target) {
                    if node.is_source {
                        all_inputs.insert(target);
                    }
                }
            }

            let mut sorted: Vec<_> = all_inputs.into_iter().collect();
            sorted.sort();
            for input in sorted {
                println!("{}", input);
            }
        }
        "restat" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let cwd = std::env::current_dir()?;
            let mut log = buildlog::BuildLog::open(&cwd);
            let mut count = 0;

            for output in graph.outputs() {
                let path = std::path::Path::new(output);
                if path.exists() {
                    if let Some(node) = graph.get_node(output) {
                        let cmd_hash = node
                            .command
                            .as_ref()
                            .map(|c| buildlog::hash_command(c))
                            .unwrap_or(0);
                        log.record(output, cmd_hash, 0, 0);
                        count += 1;
                    }
                }
            }

            if let Err(e) = log.save() {
                anyhow::bail!("Failed to save build log: {}", e);
            }
            println!("Restat'd {} outputs", count);
        }
        "cleandead" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let graph = graph::Graph::from_manifest(&manifest)?;

            let cwd = std::env::current_dir()?;
            let log = buildlog::BuildLog::open(&cwd);

            // Get all outputs currently in the manifest
            let current_outputs: std::collections::HashSet<_> =
                graph.outputs().into_iter().collect();

            let mut removed = 0;
            // Check log entries that are no longer in manifest
            for entry in log.entries() {
                if !current_outputs.contains(entry.as_str()) {
                    let path = std::path::Path::new(&entry);
                    if path.exists() {
                        if cli.verbose {
                            println!("Remove {}", entry);
                        }
                        if !cli.dry_run {
                            std::fs::remove_file(path)?;
                        }
                        removed += 1;
                    }
                }
            }

            println!("Removed {} dead outputs", removed);
        }
        "recompact" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let cwd = std::env::current_dir()?;

            // Recompact .ninja_log by reloading and saving
            let log = buildlog::BuildLog::open(&cwd);
            if let Err(e) = log.save() {
                anyhow::bail!("Failed to recompact: {}", e);
            }
            println!("Recompacted .ninja_log");
        }
        "rules" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;

            for (name, rule) in &manifest.rules {
                if cli.verbose {
                    println!("{}:", name);
                    if let Some(cmd) = &rule.command {
                        println!("  command = {}", cmd);
                    }
                    if let Some(desc) = &rule.description {
                        println!("  description = {}", desc);
                    }
                    if let Some(depfile) = &rule.depfile {
                        println!("  depfile = {}", depfile);
                    }
                } else {
                    println!("{}", name);
                }
            }
        }
        "compdb" => {
            if let Some(dir) = &cli.dir {
                std::env::set_current_dir(dir)?;
            }
            let build_file = cli.file.as_deref().unwrap_or("build.ninja");
            let manifest = parser::parse_file(std::path::Path::new(build_file))?;
            let cwd = std::env::current_dir()?;

            let mut entries = Vec::new();
            for build in &manifest.builds {
                // Get the rule
                let rule = manifest.rules.get(&build.rule);
                if let Some(rule) = rule {
                    if let Some(cmd_template) = &rule.command {
                        // Expand the command
                        let mut vars = manifest.variables.clone();
                        vars.extend(rule.variables.clone());
                        vars.extend(build.variables.clone());
                        vars.insert("in".to_string(), build.inputs.join(" "));
                        vars.insert("out".to_string(), build.outputs.join(" "));

                        let command = expand_variables(cmd_template, &vars);

                        // Create entry for each input file (typically source files)
                        for input in &build.inputs {
                            // Only include actual source files, not intermediate outputs
                            if input.ends_with(".c")
                                || input.ends_with(".cc")
                                || input.ends_with(".cpp")
                                || input.ends_with(".cxx")
                                || input.ends_with(".m")
                                || input.ends_with(".mm")
                            {
                                entries.push(serde_json::json!({
                                    "directory": cwd.to_string_lossy(),
                                    "command": command,
                                    "file": input,
                                    "output": build.outputs.first().unwrap_or(&String::new())
                                }));
                            }
                        }
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        "config" => {
            // Show config file locations and optionally generate a sample
            println!("Configuration file locations (in order of precedence):");
            println!("  1. .rninjarc (project local)");
            if let Some(home) = dirs_next::home_dir() {
                println!("  2. {}/.rninjarc", home.display());
            }
            if let Some(config_dir) = dirs_next::config_dir() {
                println!("  3. {}/rninja/config.toml", config_dir.display());
            }

            // Check if --verbose flag means generate sample config
            if cli.verbose {
                println!("\n# Sample configuration file:");
                println!("{}", config::Config::sample_config());
            } else {
                // Show current effective config
                let cfg = config::Config::load();
                println!("\nCurrent configuration:");
                println!("  Build:");
                println!("    jobs: {} (0 = num CPUs)", cfg.build.jobs);
                println!("    keep_going: {}", cfg.build.keep_going);
                println!("    explain: {}", cfg.build.explain);
                println!("  Cache:");
                println!("    enabled: {}", cfg.cache.enabled);
                println!("    mode: {}", cfg.cache.mode);
                if let Some(ref dir) = cfg.cache.directory {
                    println!("    directory: {}", dir);
                }
                println!("  Output:");
                println!("    verbose: {}", cfg.output.verbose);
                println!("    stats: {}", cfg.output.stats);
                println!("    color: {}", cfg.output.color);
                if let Some(ref trace) = cfg.output.trace_file {
                    println!("    trace_file: {}", trace);
                }
                println!("\nUse -v to show sample configuration file.");
            }
        }
        _ => {
            anyhow::bail!("unknown tool '{}'", tool);
        }
    }
    Ok(())
}

/// Expand variables in a string
fn expand_variables(input: &str, variables: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    let mut i = 0;

    while i < result.len() {
        if result[i..].starts_with('$') {
            let rest = &result[i + 1..];

            if rest.starts_with('{') {
                if let Some(end) = rest.find('}') {
                    let var_name = &rest[1..end];
                    let value = variables.get(var_name).cloned().unwrap_or_default();
                    result.replace_range(i..i + end + 2, &value);
                    i += value.len();
                    continue;
                }
            } else if rest.starts_with('$') {
                result.replace_range(i..i + 2, "$");
                i += 1;
                continue;
            } else {
                let end = rest
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                if end > 0 {
                    let var_name = &rest[..end];
                    let value = variables.get(var_name).cloned().unwrap_or_default();
                    result.replace_range(i..i + end + 1, &value);
                    i += value.len();
                    continue;
                }
            }
        }
        i += 1;
    }

    result
}
