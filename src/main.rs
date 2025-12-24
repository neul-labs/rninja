mod cache;
mod cli;
mod error;
mod executor;
mod graph;
mod parser;

use anyhow::Result;
use clap::Parser;
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
    if let Some(dir) = &cli.dir {
        std::env::set_current_dir(dir)?;
    }

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

    // Create executor and run
    let executor = executor::Executor::new(executor::Config {
        parallelism: cli.jobs.unwrap_or_else(num_cpus::get),
        dry_run: cli.dry_run,
        verbose: cli.verbose,
        keep_going: cli.keep_going,
        explain: cli.explain(),
        stats: cli.stats(),
        cache_config: cache::CacheConfig::from_env(),
    });

    executor.run(&graph, &targets)?;

    Ok(())
}

fn run_tool(tool: &str, cli: &Cli) -> Result<()> {
    match tool {
        "list" => {
            println!("rninja subtools:");
            println!("    clean    remove built files");
            println!("    commands list all commands required to rebuild given targets");
            println!("    deps     show dependencies stored in the deps log");
            println!("    graph    output graphviz dot file for targets");
            println!("    path     find dependency path between two targets");
            println!("    query    show inputs/outputs for a path");
            println!("    targets  list targets by their rule or depth in the DAG");
        }
        "clean" => {
            // TODO: Implement clean
            println!("Cleaning...");
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
        _ => {
            anyhow::bail!("unknown tool '{}'", tool);
        }
    }
    Ok(())
}
