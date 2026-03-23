// ══════════════════════════════════════════════════════════════
//  Extraction logic, file tree and minification
// ══════════════════════════════════════════════════════════════

use crate::config::extension_to_language;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// ── File tree node ────────────────────────────────────────

#[derive(Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub checked: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
}

impl FileNode {
    pub fn file_count(&self) -> usize {
        if !self.is_dir {
            return if self.checked { 1 } else { 0 };
        }
        self.children.iter().map(|c| c.file_count()).sum()
    }

    pub fn total_files(&self) -> usize {
        if !self.is_dir {
            return 1;
        }
        self.children.iter().map(|c| c.total_files()).sum()
    }

    pub fn set_checked(&mut self, state: bool) {
        self.checked = state;
        for child in &mut self.children {
            child.set_checked(state);
        }
    }

    pub fn invert_files(&mut self) {
        if !self.is_dir {
            self.checked = !self.checked;
        }
        for child in &mut self.children {
            child.invert_files();
        }
    }

    pub fn collect_checked_files(&self) -> Vec<PathBuf> {
        let mut result = Vec::new();
        if !self.is_dir && self.checked {
            result.push(self.path.clone());
        }
        for child in &self.children {
            result.extend(child.collect_checked_files());
        }
        result
    }

    /// Returns true if any node matches the search filter
    pub fn matches_search(&self, term: &str) -> bool {
        if term.is_empty() {
            return true;
        }
        let name_lower = self.name.to_lowercase();
        if !self.is_dir {
            return name_lower.contains(term);
        }
        self.children.iter().any(|c| c.matches_search(term))
    }
}

// ── Build tree from file system ────────────────────

pub fn build_tree(
    dir: &Path,
    dirs_ignorados: &HashSet<String>,
    exts_ignoradas: &HashSet<String>,
    filtro_exts: &Option<HashSet<String>>,
) -> Option<Vec<FileNode>> {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return None,
    };

    entries.sort_by(|a, b| {
        let a_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        match (a_dir, b_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
        }
    });

    let mut nodes = Vec::new();

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        if is_dir {
            if dirs_ignorados.contains(&name) {
                continue;
            }
            let children = build_tree(&path, dirs_ignorados, exts_ignoradas, filtro_exts)
                .unwrap_or_default();
            if children.is_empty() {
                continue;
            }
            nodes.push(FileNode {
                name,
                path,
                is_dir: true,
                checked: true,
                children,
                expanded: false,
            });
        } else {
            let ext = path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
                .unwrap_or_default();
            if exts_ignoradas.contains(&ext) {
                continue;
            }
            if let Some(filtro) = filtro_exts {
                if !filtro.contains(&ext) {
                    continue;
                }
            }
            nodes.push(FileNode {
                name,
                path,
                is_dir: false,
                checked: true,
                children: Vec::new(),
                expanded: false,
            });
        }
    }

    Some(nodes)
}

// ── Generate text tree ────────────────────────────

pub fn generate_tree_text(files: &[PathBuf], base: &Path) -> String {
    let mut paths: Vec<String> = files
        .iter()
        .filter_map(|f| f.strip_prefix(base).ok())
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    paths.sort();

    if paths.is_empty() {
        return String::new();
    }

    enum Entry {
        Dir(BTreeMap<String, Entry>),
    }

    fn insert(tree: &mut BTreeMap<String, Entry>, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }
        let key = parts[0].to_string();
        if parts.len() == 1 {
            tree.entry(key)
                .or_insert_with(|| Entry::Dir(BTreeMap::new()));
        } else {
            let Entry::Dir(ref mut m) = tree
                .entry(key)
                .or_insert_with(|| Entry::Dir(BTreeMap::new()));
            insert(m, &parts[1..]);
        }
    }

    let mut root = BTreeMap::new();
    for path in &paths {
        let parts: Vec<&str> = path.split('/').collect();
        insert(&mut root, &parts);
    }

    let mut lines = Vec::new();

    fn render(tree: &BTreeMap<String, Entry>, prefix: &str, lines: &mut Vec<String>) {
        let mut items: Vec<_> = tree.iter().collect();
        items.sort_by(|a, b| {
            let Entry::Dir(m) = a.1;
            let a_has = !m.is_empty();
            let Entry::Dir(m) = b.1;
            let b_has = !m.is_empty();
            match (a_has, b_has) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
            }
        });
        let total = items.len();
        for (i, (name, entry)) in items.iter().enumerate() {
            let is_last = i == total - 1;
            let connector = if is_last { "└── " } else { "├── " };
            lines.push(format!("{}{}{}", prefix, connector, name));
            let Entry::Dir(children) = entry;
            if !children.is_empty() {
                let extension = if is_last { "    " } else { "│   " };
                render(children, &format!("{}{}", prefix, extension), lines);
            }
        }
    }

    render(&root, "", &mut lines);
    lines.join("\n")
}

// ── Extract files ────────────────────────────────

pub fn extract_files(
    files: &[PathBuf],
    base: &Path,
    format: &str,
    include_tree: bool,
    progress: Option<Arc<Mutex<(usize, usize, String)>>>,
) -> (String, usize) {
    let total = files.len();
    let is_md = format == "md";
    let mut parts = Vec::new();
    let mut total_lines = 0usize;

    let project_name = base
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| base.to_string_lossy().to_string());

    // Header (will be updated later with line count)
    let header_start = parts.len();
    if is_md {
        parts.push(format!("# Project Context: {}\n\n", project_name));
        parts.push(format!("> Extracted from `{}`\n", base.display()));
        parts.push(format!("> Total files: {}\n", total));
        parts.push("LINES: [CALCULATING...]\n\n".to_string());
    } else {
        parts.push(format!("PROJECT: {}\n", project_name));
        parts.push(format!("PATH: {}\n", base.display()));
        parts.push(format!("TOTAL FILES: {}\n", total));
        parts.push("TOTAL LINES: [CALCULATING...]\n\n".to_string());
    }

    // Project structure
    if include_tree {
        let tree = generate_tree_text(files, base);
        if is_md {
            parts.push("## Project Structure\n\n".to_string());
            parts.push("```\n".to_string());
            parts.push(tree);
            parts.push("\n```\n\n".to_string());
            parts.push("---\n\n".to_string());
        } else {
            parts.push(format!("{}\n", "=".repeat(60)));
            parts.push("PROJECT STRUCTURE\n".to_string());
            parts.push(format!("{}\n\n", "=".repeat(60)));
            parts.push(tree);
            parts.push(format!("\n\n{}\n\n", "=".repeat(60)));
        }
    }

    // Files
    for (i, path) in files.iter().enumerate() {
        let relative_path = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if let Some(ref prog) = progress {
            if let Ok(mut p) = prog.lock() {
                *p = (i + 1, total, relative_path.clone());
            }
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let num_lines = content.lines().count();
                total_lines += num_lines;

                if is_md {
                    let ext = path
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
                        .unwrap_or_default();
                    let lang = extension_to_language(&ext);
                    parts.push(format!("## `{}`\n\n", relative_path));
                    parts.push(format!("```{}\n", lang));
                    parts.push(content.clone());
                    if !content.ends_with('\n') {
                        parts.push("\n".to_string());
                    }
                    parts.push("```\n\n".to_string());
                } else {
                    parts.push(format!("\n{}\n", "=".repeat(60)));
                    parts.push(format!("FILE: {}\n", relative_path));
                    parts.push(format!("{}\n\n", "=".repeat(60)));
                    parts.push(content);
                    parts.push("\n".to_string());
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("invalid utf-8")
                    || err_str.contains("stream did not contain valid UTF-8")
                {
                    if is_md {
                        parts.push(format!(
                            "## `{}` - BINARY IGNORED\n\n",
                            relative_path
                        ));
                    } else {
                        parts.push(format!("\n{}\n", "=".repeat(60)));
                        parts.push(format!(
                            "FILE: {} [BINARY IGNORED]\n",
                            relative_path
                        ));
                        parts.push(format!("{}\n\n", "=".repeat(60)));
                    }
                } else {
                    parts.push(format!(
                        "\n[ERROR READING {}]: {}\n",
                        relative_path, e
                    ));
                }
            }
        }
    }

    // Update header with line count
    let line_info = if is_md {
        format!("> Total lines: {}\n\n", total_lines)
    } else {
        format!("TOTAL LINES: {}\n\n", total_lines)
    };
    parts[header_start + 3] = line_info;

    (parts.join(""), total_lines)
}

// ── Minify content for LLM ─────────────────────────

pub fn minify(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut previous_empty_line = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip consecutive empty lines
        if trimmed.is_empty() {
            if !previous_empty_line {
                result.push('\n');
                previous_empty_line = true;
            }
            continue;
        }
        previous_empty_line = false;

        // Skip decorative lines (only ====, ----, etc.)
        if trimmed
            .chars()
            .all(|c| c == '=' || c == '-' || c == '─' || c == '━')
            && trimmed.len() > 3
        {
            continue;
        }

        // Remove excessive indentation: keep only 1 space per tab/4 spaces
        let leading_spaces = line.len() - line.trim_start().len();
        let indent_level = leading_spaces / 4;
        if indent_level > 0 {
            for _ in 0..indent_level {
                result.push(' ');
            }
        }

        result.push_str(trimmed);
        result.push('\n');
    }

    result
}

