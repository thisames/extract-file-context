// ══════════════════════════════════════════════════════════════
//  Configurações e constantes
// ══════════════════════════════════════════════════════════════

use std::collections::HashSet;

// ── Mapeamento extensão -> linguagem para blocos Markdown ───

pub fn extensao_linguagem(ext: &str) -> &'static str {
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

// ── Constantes padrão ───────────────────────────────────────

pub fn dirs_ignorados_padrao() -> Vec<String> {
    vec![
        ".git", ".vscode", ".idea", "__pycache__", "node_modules",
        ".venv", "env", "venv", "build", "dist", ".next", ".nuxt",
        ".cache", ".pytest_cache", ".mypy_cache", "coverage",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn extensoes_ignoradas_padrao() -> Vec<String> {
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

// ── Definição de linguagens para filtro ─────────────────────

pub struct LinguagemDef {
    pub nome: &'static str,
    pub emoji: &'static str,
    pub extensoes: &'static [&'static str],
}

pub const LINGUAGENS: &[LinguagemDef] = &[
    LinguagemDef { nome: "Python",       emoji: "🐍", extensoes: &[".py"] },
    LinguagemDef { nome: "JavaScript",   emoji: "🟨", extensoes: &[".js", ".jsx"] },
    LinguagemDef { nome: "TypeScript",   emoji: "🔷", extensoes: &[".ts", ".tsx"] },
    LinguagemDef { nome: "Rust",         emoji: "🦀", extensoes: &[".rs"] },
    LinguagemDef { nome: "Java",         emoji: "☕", extensoes: &[".java"] },
    LinguagemDef { nome: "Kotlin",       emoji: "🟣", extensoes: &[".kt"] },
    LinguagemDef { nome: "Go",           emoji: "🐹", extensoes: &[".go"] },
    LinguagemDef { nome: "C",            emoji: "🔵", extensoes: &[".c", ".h"] },
    LinguagemDef { nome: "C++",          emoji: "🔵", extensoes: &[".cpp", ".hpp"] },
    LinguagemDef { nome: "C#",           emoji: "🟢", extensoes: &[".cs"] },
    LinguagemDef { nome: "Swift",        emoji: "🍎", extensoes: &[".swift"] },
    LinguagemDef { nome: "Dart",         emoji: "🎯", extensoes: &[".dart"] },
    LinguagemDef { nome: "Ruby",         emoji: "💎", extensoes: &[".rb"] },
    LinguagemDef { nome: "PHP",          emoji: "🐘", extensoes: &[".php"] },
    LinguagemDef { nome: "Lua",          emoji: "🌙", extensoes: &[".lua"] },
    LinguagemDef { nome: "R",            emoji: "📊", extensoes: &[".r"] },
    LinguagemDef { nome: "Perl",         emoji: "🐪", extensoes: &[".perl", ".pl"] },
    LinguagemDef { nome: "Groovy",       emoji: "⭐", extensoes: &[".groovy", ".gradle"] },
    LinguagemDef { nome: "HTML",         emoji: "🌐", extensoes: &[".html"] },
    LinguagemDef { nome: "CSS/SCSS",     emoji: "🎨", extensoes: &[".css", ".scss", ".less"] },
    LinguagemDef { nome: "SQL",          emoji: "🗃️", extensoes: &[".sql"] },
    LinguagemDef { nome: "Shell/Bash",   emoji: "🐚", extensoes: &[".sh", ".bash", ".zsh", ".fish"] },
    LinguagemDef { nome: "JSON",         emoji: "📋", extensoes: &[".json"] },
    LinguagemDef { nome: "YAML",         emoji: "📝", extensoes: &[".yaml", ".yml"] },
    LinguagemDef { nome: "TOML",         emoji: "⚙️", extensoes: &[".toml"] },
    LinguagemDef { nome: "XML",          emoji: "📰", extensoes: &[".xml"] },
    LinguagemDef { nome: "Markdown",     emoji: "📖", extensoes: &[".md"] },
    LinguagemDef { nome: "Vue",          emoji: "💚", extensoes: &[".vue"] },
    LinguagemDef { nome: "Svelte",       emoji: "🧡", extensoes: &[".svelte"] },
    LinguagemDef { nome: "Dockerfile",   emoji: "🐳", extensoes: &[".dockerfile"] },
    LinguagemDef { nome: "Terraform",    emoji: "🏗️", extensoes: &[".tf"] },
    LinguagemDef { nome: "INI/Config",   emoji: "🔧", extensoes: &[".ini", ".cfg"] },
];

/// Retorna o conjunto de extensões das linguagens selecionadas
pub fn extensoes_das_linguagens(selecionadas: &[bool]) -> HashSet<String> {
    let mut exts = HashSet::new();
    for (i, lang) in LINGUAGENS.iter().enumerate() {
        if i < selecionadas.len() && selecionadas[i] {
            for ext in lang.extensoes {
                exts.insert(ext.to_string());
            }
        }
    }
    exts
}

