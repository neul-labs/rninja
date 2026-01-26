mod lexer;
mod manifest;

pub use manifest::{Build, Manifest, Pool, Rule};

use crate::error::ParseError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Parse a ninja build file and return the manifest
pub fn parse(content: &str) -> Result<Manifest, ParseError> {
    parse_with_base(content, None)
}

/// Parse a ninja build file with a base directory for includes
pub fn parse_file(path: &Path) -> Result<Manifest, ParseError> {
    let content = std::fs::read_to_string(path).map_err(|e| ParseError::Syntax {
        line: 0,
        message: format!("failed to read {}: {}", path.display(), e),
    })?;

    let base_dir = path.parent().map(|p| p.to_path_buf());
    parse_with_base(&content, base_dir)
}

fn parse_with_base(content: &str, base_dir: Option<PathBuf>) -> Result<Manifest, ParseError> {
    let mut parser = Parser::new(content, base_dir);
    parser.parse()
}

struct Parser<'a> {
    lexer: lexer::Lexer<'a>,
    manifest: Manifest,
    current_line: usize,
    base_dir: Option<PathBuf>,
    included_files: HashSet<PathBuf>,
}

impl<'a> Parser<'a> {
    fn new(content: &'a str, base_dir: Option<PathBuf>) -> Self {
        Self {
            lexer: lexer::Lexer::new(content),
            manifest: Manifest::default(),
            current_line: 1,
            base_dir,
            included_files: HashSet::new(),
        }
    }

    fn parse(&mut self) -> Result<Manifest, ParseError> {
        while let Some(line) = self.lexer.next_line() {
            self.current_line = self.lexer.line_number();

            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(rest) = line.strip_prefix("rule ") {
                self.parse_rule(rest.trim())?;
            } else if let Some(rest) = line.strip_prefix("build ") {
                self.parse_build(rest.trim())?;
            } else if let Some(rest) = line.strip_prefix("default ") {
                self.parse_default(rest.trim())?;
            } else if let Some(rest) = line.strip_prefix("pool ") {
                self.parse_pool(rest.trim())?;
            } else if let Some(rest) = line.strip_prefix("include ") {
                self.parse_include(rest.trim())?;
            } else if let Some(rest) = line.strip_prefix("subninja ") {
                self.parse_subninja(rest.trim())?;
            } else if line.contains('=') {
                self.parse_variable(line)?;
            } else {
                return Err(ParseError::Syntax {
                    line: self.current_line,
                    message: format!("unexpected line: {}", line),
                });
            }
        }

        Ok(std::mem::take(&mut self.manifest))
    }

    fn parse_rule(&mut self, name: &str) -> Result<(), ParseError> {
        if self.manifest.rules.contains_key(name) {
            return Err(ParseError::DuplicateRule {
                name: name.to_string(),
            });
        }

        let mut rule = Rule {
            name: name.to_string(),
            ..Default::default()
        };

        // Parse indented variables
        while let Some(line) = self.lexer.peek_line() {
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            let line = self.lexer.next_line().ok_or_else(|| ParseError::Syntax {
                line: self.current_line,
                message: "unexpected end of file while parsing rule".to_string(),
            })?;
            self.current_line = self.lexer.line_number();

            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "command" => rule.command = Some(value.to_string()),
                    "description" => rule.description = Some(value.to_string()),
                    "depfile" => rule.depfile = Some(value.to_string()),
                    "deps" => rule.deps = Some(value.to_string()),
                    "generator" => rule.generator = value == "1" || value == "true",
                    "restat" => rule.restat = value == "1" || value == "true",
                    "rspfile" => rule.rspfile = Some(value.to_string()),
                    "rspfile_content" => rule.rspfile_content = Some(value.to_string()),
                    "pool" => rule.pool = Some(value.to_string()),
                    _ => {
                        // Store unknown variables for later expansion
                        rule.variables.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        self.manifest.rules.insert(name.to_string(), rule);
        Ok(())
    }

    fn parse_build(&mut self, line: &str) -> Result<(), ParseError> {
        // Format: outputs: rule inputs | implicit_deps || order_only_deps
        let (outputs_part, rest) = line.split_once(':').ok_or_else(|| ParseError::Syntax {
            line: self.current_line,
            message: "build line missing ':'".to_string(),
        })?;

        let outputs: Vec<String> = outputs_part
            .split_whitespace()
            .map(|s| self.expand_path(s))
            .collect();

        if outputs.is_empty() {
            return Err(ParseError::Syntax {
                line: self.current_line,
                message: "build line has no outputs".to_string(),
            });
        }

        let rest = rest.trim();
        let mut parts = rest.splitn(2, char::is_whitespace);
        let rule = parts.next().unwrap_or("").to_string();
        let deps_str = parts.next().unwrap_or("");

        // Parse dependencies: inputs | implicit || order_only
        let (explicit, rest) = deps_str.split_once("||").map_or((deps_str, ""), |(a, b)| (a, b));
        let order_only: Vec<String> = rest
            .split_whitespace()
            .map(|s| self.expand_path(s))
            .collect();

        let (explicit, implicit_str) = explicit.split_once('|').unwrap_or((explicit, ""));
        let inputs: Vec<String> = explicit
            .split_whitespace()
            .map(|s| self.expand_path(s))
            .collect();
        let implicit: Vec<String> = implicit_str
            .split_whitespace()
            .map(|s| self.expand_path(s))
            .collect();

        let mut build = Build {
            outputs,
            rule,
            inputs,
            implicit_deps: implicit,
            order_only_deps: order_only,
            ..Default::default()
        };

        // Parse indented variables (build-specific overrides)
        while let Some(line) = self.lexer.peek_line() {
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            let line = self.lexer.next_line().ok_or_else(|| ParseError::Syntax {
                line: self.current_line,
                message: "unexpected end of file while parsing build".to_string(),
            })?;
            self.current_line = self.lexer.line_number();

            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                build
                    .variables
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        self.manifest.builds.push(build);
        Ok(())
    }

    fn parse_default(&mut self, line: &str) -> Result<(), ParseError> {
        for target in line.split_whitespace() {
            self.manifest.defaults.push(self.expand_path(target));
        }
        Ok(())
    }

    fn parse_pool(&mut self, name: &str) -> Result<(), ParseError> {
        let mut pool = Pool {
            name: name.to_string(),
            depth: 1,
        };

        // Parse indented variables
        while let Some(line) = self.lexer.peek_line() {
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            let line = self.lexer.next_line().ok_or_else(|| ParseError::Syntax {
                line: self.current_line,
                message: "unexpected end of file while parsing pool".to_string(),
            })?;
            self.current_line = self.lexer.line_number();

            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "depth" {
                    pool.depth = value.trim().parse().unwrap_or(1);
                }
            }
        }

        self.manifest.pools.insert(name.to_string(), pool);
        Ok(())
    }

    fn parse_include(&mut self, path: &str) -> Result<(), ParseError> {
        let path = self.expand_path(path);
        let full_path = self.resolve_path(&path);

        // Check for circular includes
        if self.included_files.contains(&full_path) {
            return Err(ParseError::CircularInclude {
                path: full_path.display().to_string(),
            });
        }

        // Read and parse the included file
        let content = std::fs::read_to_string(&full_path).map_err(|e| ParseError::Syntax {
            line: self.current_line,
            message: format!("failed to include {}: {}", full_path.display(), e),
        })?;

        self.included_files.insert(full_path.clone());
        self.manifest.includes.push(path);

        // Parse included content into the same manifest (same scope)
        let mut sub_parser = Parser {
            lexer: lexer::Lexer::new(&content),
            manifest: std::mem::take(&mut self.manifest),
            current_line: 1,
            base_dir: full_path.parent().map(|p| p.to_path_buf()),
            included_files: self.included_files.clone(),
        };

        self.manifest = sub_parser.parse()?;
        self.included_files = sub_parser.included_files;

        Ok(())
    }

    fn parse_subninja(&mut self, path: &str) -> Result<(), ParseError> {
        let path = self.expand_path(path);
        let full_path = self.resolve_path(&path);

        // Check for circular includes
        if self.included_files.contains(&full_path) {
            return Err(ParseError::CircularInclude {
                path: full_path.display().to_string(),
            });
        }

        // Read and parse the subninja file
        let content = std::fs::read_to_string(&full_path).map_err(|e| ParseError::Syntax {
            line: self.current_line,
            message: format!("failed to include subninja {}: {}", full_path.display(), e),
        })?;

        self.included_files.insert(full_path.clone());
        self.manifest.subninjas.push(path);

        // Parse subninja content with separate scope (new variables, but shares rules/pools/builds)
        let mut sub_manifest = Manifest {
            rules: self.manifest.rules.clone(),
            pools: self.manifest.pools.clone(),
            ..Default::default()
        };

        let mut sub_parser = Parser {
            lexer: lexer::Lexer::new(&content),
            manifest: sub_manifest,
            current_line: 1,
            base_dir: full_path.parent().map(|p| p.to_path_buf()),
            included_files: self.included_files.clone(),
        };

        sub_manifest = sub_parser.parse()?;
        self.included_files = sub_parser.included_files;

        // Merge results back: builds, defaults, new rules, new pools
        self.manifest.builds.extend(sub_manifest.builds);
        self.manifest.defaults.extend(sub_manifest.defaults);
        for (k, v) in sub_manifest.rules {
            self.manifest.rules.entry(k).or_insert(v);
        }
        for (k, v) in sub_manifest.pools {
            self.manifest.pools.entry(k).or_insert(v);
        }

        Ok(())
    }

    fn parse_variable(&mut self, line: &str) -> Result<(), ParseError> {
        if let Some((key, value)) = line.split_once('=') {
            self.manifest
                .variables
                .insert(key.trim().to_string(), value.trim().to_string());
        }
        Ok(())
    }

    fn expand_path(&self, path: &str) -> String {
        // Handle $var and ${var} expansion for path variables
        // Note: Full variable expansion (including $in, $out, etc.) happens later
        // in the graph construction phase. This only expands manifest-level
        // global variables for path components.
        let mut result = path.to_string();
        for (key, value) in &self.manifest.variables {
            result = result.replace(&format!("${}", key), value);
            result = result.replace(&format!("${{{}}}", key), value);
        }
        result
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else if let Some(base) = &self.base_dir {
            base.join(p)
        } else {
            p.to_path_buf()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let content = r#"
rule cc
    command = gcc -c $in -o $out
    description = Compiling $in
"#;
        let manifest = parse(content).unwrap();
        assert!(manifest.rules.contains_key("cc"));
        let rule = &manifest.rules["cc"];
        assert_eq!(rule.command, Some("gcc -c $in -o $out".to_string()));
    }

    #[test]
    fn test_parse_build() {
        let content = r#"
rule cc
    command = gcc -c $in -o $out

build foo.o: cc foo.c | header.h
"#;
        let manifest = parse(content).unwrap();
        assert_eq!(manifest.builds.len(), 1);
        let build = &manifest.builds[0];
        assert_eq!(build.outputs, vec!["foo.o"]);
        assert_eq!(build.rule, "cc");
        assert_eq!(build.inputs, vec!["foo.c"]);
        assert_eq!(build.implicit_deps, vec!["header.h"]);
    }

    #[test]
    fn test_parse_default() {
        let content = r#"
default foo bar
"#;
        let manifest = parse(content).unwrap();
        assert_eq!(manifest.defaults, vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_pool() {
        let content = r#"
pool link_pool
    depth = 4
"#;
        let manifest = parse(content).unwrap();
        assert!(manifest.pools.contains_key("link_pool"));
        assert_eq!(manifest.pools["link_pool"].depth, 4);
    }

    #[test]
    fn test_parse_variables() {
        let content = r#"
builddir = out
cflags = -Wall

rule cc
    command = gcc $cflags -c $in -o $out

build $builddir/foo.o: cc foo.c
"#;
        let manifest = parse(content).unwrap();
        assert_eq!(manifest.variables.get("builddir"), Some(&"out".to_string()));
        assert_eq!(manifest.builds[0].outputs, vec!["out/foo.o"]);
    }
}
