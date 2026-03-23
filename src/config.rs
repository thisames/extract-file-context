// ══════════════════════════════════════════════════════════════
//  Configuration and constants
// ══════════════════════════════════════════════════════════════

use std::collections::HashSet;

// ── Extension -> language mapping for Markdown blocks ──

pub fn extension_to_language(ext: &str) -> &'static str {
    match ext {
        ".py" => "python",
        ".js" => "javascript",
        ".ts" => "typescript",
        ".jsx" => "jsx",
        ".tsx" => "tsx",
        ".java" => "java",
        ".kt" => "kotlin",
        ".go" => "go",
        ".rs" => "rust",
        ".rb" => "ruby",
        ".php" => "php",
        ".c" => "c",
        ".cpp" | ".hpp" => "cpp",
        ".h" => "c",
        ".cs" => "csharp",
        ".swift" => "swift",
        ".dart" => "dart",
        ".html" => "html",
        ".css" => "css",
        ".scss" => "scss",
        ".less" => "less",
        ".json" => "json",
        ".xml" => "xml",
        ".yaml" | ".yml" => "yaml",
        ".toml" => "toml",
        ".ini" | ".cfg" => "ini",
        ".sh" | ".bash" => "bash",
        ".zsh" => "zsh",
        ".fish" => "fish",
        ".sql" => "sql",
        ".md" => "markdown",
        ".r" => "r",
        ".vue" => "vue",
        ".svelte" => "svelte",
        ".dockerfile" => "dockerfile",
        ".tf" => "hcl",
        ".lua" => "lua",
        ".perl" | ".pl" => "perl",
        ".gradle" | ".groovy" => "groovy",
        _ => "",
    }
}

// ── Default constants ───────────────────────────────────────

pub fn default_ignored_dirs() -> Vec<String> {
    vec![
        ".git", ".vscode", ".idea", "__pycache__", "node_modules",
        ".venv", "env", "venv", "build", "dist", ".next", ".nuxt",
        ".cache", ".pytest_cache", ".mypy_cache", "coverage",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn default_ignored_extensions() -> Vec<String> {
    vec![
        ".pyc", ".pyo", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".webp",
        ".exe", ".dll", ".so", ".dylib", ".pdf", ".zip", ".tar", ".gz", ".rar",
        ".mp4", ".mp3", ".wav", ".avi", ".mov",
        ".sqlite3", ".db", ".lock", ".map",
        ".woff", ".woff2", ".ttf", ".eot",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// ── Language definitions for filter ─────────────────────────

pub struct LanguageDefinition {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

pub const LANGUAGES: &[LanguageDefinition] = &[
    LanguageDefinition { name: "Python",       extensions: &[".py"] },
    LanguageDefinition { name: "JavaScript",   extensions: &[".js", ".jsx"] },
    LanguageDefinition { name: "TypeScript",   extensions: &[".ts", ".tsx"] },
    LanguageDefinition { name: "Rust",         extensions: &[".rs"] },
    LanguageDefinition { name: "Java",         extensions: &[".java"] },
    LanguageDefinition { name: "Kotlin",       extensions: &[".kt"] },
    LanguageDefinition { name: "Go",           extensions: &[".go"] },
    LanguageDefinition { name: "C",            extensions: &[".c", ".h"] },
    LanguageDefinition { name: "C++",          extensions: &[".cpp", ".hpp"] },
    LanguageDefinition { name: "C#",           extensions: &[".cs"] },
    LanguageDefinition { name: "Swift",        extensions: &[".swift"] },
    LanguageDefinition { name: "Dart",         extensions: &[".dart"] },
    LanguageDefinition { name: "Ruby",         extensions: &[".rb"] },
    LanguageDefinition { name: "PHP",          extensions: &[".php"] },
    LanguageDefinition { name: "Lua",          extensions: &[".lua"] },
    LanguageDefinition { name: "R",            extensions: &[".r"] },
    LanguageDefinition { name: "Perl",         extensions: &[".perl", ".pl"] },
    LanguageDefinition { name: "Groovy",       extensions: &[".groovy", ".gradle"] },
    LanguageDefinition { name: "HTML",         extensions: &[".html"] },
    LanguageDefinition { name: "CSS/SCSS",     extensions: &[".css", ".scss", ".less"] },
    LanguageDefinition { name: "SQL",          extensions: &[".sql"] },
    LanguageDefinition { name: "Shell/Bash",   extensions: &[".sh", ".bash", ".zsh", ".fish"] },
    LanguageDefinition { name: "JSON",         extensions: &[".json"] },
    LanguageDefinition { name: "YAML",         extensions: &[".yaml", ".yml"] },
    LanguageDefinition { name: "TOML",         extensions: &[".toml"] },
    LanguageDefinition { name: "XML",          extensions: &[".xml"] },
    LanguageDefinition { name: "Markdown",     extensions: &[".md"] },
    LanguageDefinition { name: "Vue",          extensions: &[".vue"] },
    LanguageDefinition { name: "Svelte",       extensions: &[".svelte"] },
    LanguageDefinition { name: "Dockerfile",   extensions: &[".dockerfile"] },
    LanguageDefinition { name: "Terraform",    extensions: &[".tf"] },
    LanguageDefinition { name: "INI/Config",   extensions: &[".ini", ".cfg"] },
];

/// Returns the set of extensions for the selected languages
pub fn extensions_from_languages(selected: &[bool]) -> HashSet<String> {
    let mut exts = HashSet::new();
    for (i, lang) in LANGUAGES.iter().enumerate() {
        if i < selected.len() && selected[i] {
            for ext in lang.extensions {
                exts.insert(ext.to_string());
            }
        }
    }
    exts
}

