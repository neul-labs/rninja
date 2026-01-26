mod node;

pub use node::Node;

use crate::error::GraphError;
use crate::parser::{Build, Manifest, Rule};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// Hash a command string for quick comparison
#[inline]
fn hash_command(cmd: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    cmd.hash(&mut hasher);
    hasher.finish()
}

/// The build graph - a DAG of nodes representing build targets
#[derive(Debug)]
pub struct Graph {
    /// All nodes indexed by output path
    nodes: HashMap<String, Node>,
    /// Rules from the manifest
    rules: HashMap<String, Rule>,
    /// Global variables for expansion
    variables: HashMap<String, String>,
    /// Pool depths for parallelism limiting
    pools: HashMap<String, usize>,
}

impl Graph {
    /// Build a graph from a parsed manifest
    pub fn from_manifest(manifest: &Manifest) -> Result<Self, GraphError> {
        let mut graph = Self {
            nodes: HashMap::new(),
            rules: manifest.rules.clone(),
            variables: manifest.variables.clone(),
            pools: manifest.pools.iter().map(|(k, v)| (k.clone(), v.depth)).collect(),
        };

        // First pass: create nodes for all build outputs
        for build in &manifest.builds {
            let rule = manifest.rules.get(&build.rule);
            let command = graph.expand_command(build, rule);
            let description = graph.expand_description(build, rule);

            // Pre-compute command hash for fast up-to-date checking
            let command_hash = command.as_ref().map(|c| hash_command(c)).unwrap_or(0);

            for output in build.all_outputs() {
                if graph.nodes.contains_key(output) {
                    return Err(GraphError::DuplicateOutput {
                        output: output.to_string(),
                    });
                }

                let node = Node {
                    path: output.to_string(),
                    command: command.clone(),
                    command_hash,
                    description: description.clone(),
                    deps: build.all_deps().map(|s| s.to_string()).collect(),
                    rule: build.rule.clone(),
                    is_phony: build.rule == "phony",
                    is_source: false,
                    restat: rule.map(|r| r.restat).unwrap_or(false),
                    depfile: graph.expand_var(
                        rule.and_then(|r| r.depfile.as_deref()).unwrap_or(""),
                        build,
                        rule,
                    ),
                    pool: rule.and_then(|r| r.pool.clone()),
                    generator: rule.map(|r| r.generator).unwrap_or(false),
                    rspfile: rule.and_then(|r| r.rspfile.as_deref()).map(|s| {
                        graph.expand_var(s, build, rule)
                    }),
                    rspfile_content: rule.and_then(|r| r.rspfile_content.as_deref()).map(|s| {
                        graph.expand_var(s, build, rule)
                    }),
                };

                graph.nodes.insert(output.to_string(), node);
            }
        }

        // Second pass: create implicit nodes for source files (files with no build rule)
        let mut source_files = HashSet::new();
        for node in graph.nodes.values() {
            for dep in &node.deps {
                if !graph.nodes.contains_key(dep) {
                    source_files.insert(dep.clone());
                }
            }
        }

        for source in source_files {
            graph.nodes.insert(
                source.clone(),
                Node {
                    path: source,
                    is_source: true,
                    ..Default::default()
                },
            );
        }

        // Validate: check for cycles
        graph.check_cycles()?;

        Ok(graph)
    }

    /// Get a node by path
    pub fn get_node(&self, path: &str) -> Option<&Node> {
        self.nodes.get(path)
    }

    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Get pool depths for parallelism limiting
    pub fn pool_depths(&self) -> HashMap<String, usize> {
        self.pools.clone()
    }

    /// Get nodes in topological order for given targets
    pub fn topo_order(&self, targets: &[&str]) -> Result<Vec<&Node>, GraphError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for target in targets {
            self.topo_visit(target, &mut visited, &mut in_stack, &mut result)?;
        }

        Ok(result)
    }

    /// Get topological order as paths (for compatibility)
    pub fn topological_order(&self, targets: &[&str]) -> Result<Vec<String>, GraphError> {
        let nodes = self.topo_order(targets)?;
        Ok(nodes.iter().map(|n| n.path.clone()).collect())
    }

    /// Get all built outputs (non-source nodes)
    pub fn outputs(&self) -> Vec<&str> {
        self.nodes
            .values()
            .filter(|n| !n.is_source)
            .map(|n| n.path.as_str())
            .collect()
    }

    /// Find the shortest path between two nodes
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to.to_string()];
                let mut node = to.to_string();
                while let Some(p) = parent.get(&node) {
                    path.push(p.clone());
                    node = p.clone();
                }
                path.reverse();
                return Some(path);
            }

            if let Some(n) = self.nodes.get(&current) {
                for dep in &n.deps {
                    if !visited.contains(dep) {
                        visited.insert(dep.clone());
                        parent.insert(dep.clone(), current.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        None
    }

    /// Generate GraphViz DOT format for targets
    pub fn to_dot(&self, targets: &[&str]) -> String {
        let mut lines = vec![
            "digraph ninja {".to_string(),
            "  rankdir=BT;".to_string(),
            "  node [shape=box];".to_string(),
        ];

        // Collect all nodes reachable from targets
        let mut to_visit: Vec<&str> = targets.to_vec();
        let mut visited = HashSet::new();

        while let Some(path) = to_visit.pop() {
            if visited.contains(path) {
                continue;
            }
            visited.insert(path.to_string());

            if let Some(node) = self.nodes.get(path) {
                let node_id = escape_dot_id(path);
                let shape = if node.is_source { "ellipse" } else { "box" };
                lines.push(format!("  {} [shape={}, label=\"{}\"];", node_id, shape, escape_dot_label(path)));

                for dep in &node.deps {
                    let dep_id = escape_dot_id(dep);
                    lines.push(format!("  {} -> {};", dep_id, node_id));
                    to_visit.push(dep);
                }
            }
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Get inputs for a target
    pub fn inputs_for(&self, target: &str) -> Option<&Vec<String>> {
        self.nodes.get(target).map(|n| &n.deps)
    }

    /// Get all outputs that depend on a given target
    pub fn outputs_for(&self, target: &str) -> Vec<&str> {
        self.nodes
            .values()
            .filter(|n| n.deps.contains(&target.to_string()))
            .map(|n| n.path.as_str())
            .collect()
    }

    fn topo_visit<'a>(
        &'a self,
        target: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        result: &mut Vec<&'a Node>,
    ) -> Result<(), GraphError> {
        if visited.contains(target) {
            return Ok(());
        }

        if in_stack.contains(target) {
            return Err(GraphError::Cycle {
                target: target.to_string(),
            });
        }

        let node = self.nodes.get(target).ok_or_else(|| GraphError::UnknownTarget {
            target: target.to_string(),
        })?;

        in_stack.insert(target.to_string());

        for dep in &node.deps {
            self.topo_visit(dep, visited, in_stack, result)?;
        }

        in_stack.remove(target);
        visited.insert(target.to_string());
        result.push(node);

        Ok(())
    }

    fn check_cycles(&self) -> Result<(), GraphError> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for path in self.nodes.keys() {
            self.check_cycles_visit(path, &mut visited, &mut in_stack)?;
        }

        Ok(())
    }

    fn check_cycles_visit(
        &self,
        path: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
    ) -> Result<(), GraphError> {
        if visited.contains(path) {
            return Ok(());
        }

        if in_stack.contains(path) {
            return Err(GraphError::Cycle {
                target: path.to_string(),
            });
        }

        in_stack.insert(path.to_string());

        if let Some(node) = self.nodes.get(path) {
            for dep in &node.deps {
                self.check_cycles_visit(dep, visited, in_stack)?;
            }
        }

        in_stack.remove(path);
        visited.insert(path.to_string());

        Ok(())
    }

    /// Expand the command for a build edge
    fn expand_command(&self, build: &Build, rule: Option<&Rule>) -> Option<String> {
        let cmd = rule?.command.as_ref()?;
        Some(self.expand_var(cmd, build, rule))
    }

    /// Expand the description for a build edge
    fn expand_description(&self, build: &Build, rule: Option<&Rule>) -> Option<String> {
        let desc = rule?.description.as_ref()?;
        Some(self.expand_var(desc, build, rule))
    }

    /// Expand variables in a string
    ///
    /// Variable expansion order (later sources override earlier ones):
    /// 1. Built-in variables ($in, $out, $in_newline, $first_input, $first_output)
    /// 2. Build-specific variables (from build.ninja build statements)
    /// 3. Rule variables (from rule definitions in build.ninja)
    /// 4. Global variables (from top-level assignments in build.ninja)
    /// 5. Escape sequences ($$ -> $, $space, $:)
    ///
    /// Note: Expansion is single-pass. Variable values containing $var references
    /// will not be recursively expanded. Use unique variable names to avoid conflicts.
    fn expand_var(&self, s: &str, build: &Build, rule: Option<&Rule>) -> String {
        let mut result = s.to_string();

        // 1. Built-in variables
        let in_str = build.inputs.join(" ");
        let out_str = build.outputs.join(" ");
        let in_first = build.inputs.first().map(|s| s.as_str()).unwrap_or("");
        let out_first = build.outputs.first().map(|s| s.as_str()).unwrap_or("");

        result = result.replace("$in", &in_str);
        result = result.replace("${in}", &in_str);
        result = result.replace("$out", &out_str);
        result = result.replace("${out}", &out_str);
        result = result.replace("$in_newline", &build.inputs.join("\n"));
        result = result.replace("$first_input", in_first);
        result = result.replace("$first_output", out_first);

        // 2. Build-specific variables (highest priority - can override rule/global vars)
        for (key, value) in &build.variables {
            result = result.replace(&format!("${}", key), value);
            result = result.replace(&format!("${{{}}}", key), value);
        }

        // 3. Rule variables
        if let Some(rule) = rule {
            for (key, value) in &rule.variables {
                result = result.replace(&format!("${}", key), value);
                result = result.replace(&format!("${{{}}}", key), value);
            }
        }

        // 4. Global variables (lowest priority custom variables)
        for (key, value) in &self.variables {
            result = result.replace(&format!("${}", key), value);
            result = result.replace(&format!("${{{}}}", key), value);
        }

        // 5. Escape sequences (processed last)
        result = result.replace("$$", "$");
        result = result.replace("$ ", " ");
        result = result.replace("$:", ":");

        result
    }
}

fn escape_dot_id(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn escape_dot_label(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_simple_graph() {
        let content = r#"
rule cc
    command = gcc -c $in -o $out

build foo.o: cc foo.c
build bar.o: cc bar.c
build prog: phony foo.o bar.o
"#;
        let manifest = parser::parse(content).unwrap();
        let graph = Graph::from_manifest(&manifest).unwrap();

        assert!(graph.get_node("foo.o").is_some());
        assert!(graph.get_node("bar.o").is_some());
        assert!(graph.get_node("prog").is_some());
        assert!(graph.get_node("foo.c").is_some()); // source file
    }

    #[test]
    fn test_topo_order() {
        let content = r#"
rule cc
    command = gcc -c $in -o $out
rule link
    command = gcc $in -o $out

build foo.o: cc foo.c
build bar.o: cc bar.c
build prog: link foo.o bar.o
"#;
        let manifest = parser::parse(content).unwrap();
        let graph = Graph::from_manifest(&manifest).unwrap();

        let order = graph.topo_order(&["prog"]).unwrap();
        let paths: Vec<&str> = order.iter().map(|n| n.path.as_str()).collect();

        // prog should come after foo.o and bar.o
        let prog_idx = paths.iter().position(|&p| p == "prog").unwrap();
        let foo_idx = paths.iter().position(|&p| p == "foo.o").unwrap();
        let bar_idx = paths.iter().position(|&p| p == "bar.o").unwrap();

        assert!(foo_idx < prog_idx);
        assert!(bar_idx < prog_idx);
    }

    #[test]
    fn test_cycle_detection() {
        let content = r#"
rule cc
    command = echo

build a: cc b
build b: cc c
build c: cc a
"#;
        let manifest = parser::parse(content).unwrap();
        let result = Graph::from_manifest(&manifest);
        assert!(matches!(result, Err(GraphError::Cycle { .. })));
    }
}
