// ══════════════════════════════════════════════════════════════
//  Lógica de extração, árvore de arquivos e minificação
// ══════════════════════════════════════════════════════════════

use crate::config::extensao_linguagem;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// ── Nó da árvore de arquivos ────────────────────────────────

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

    /// Retorna true se algum nó casa com o filtro de busca
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

// ── Construir árvore a partir do sistema de arquivos ────────

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

// ── Gerar árvore textual ────────────────────────────────────

pub fn gerar_arvore_texto(arquivos: &[PathBuf], base: &Path) -> String {
    let mut caminhos: Vec<String> = arquivos
        .iter()
        .filter_map(|f| f.strip_prefix(base).ok())
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    caminhos.sort();

    if caminhos.is_empty() {
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
    for caminho in &caminhos {
        let parts: Vec<&str> = caminho.split('/').collect();
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

// ── Extrair arquivos ────────────────────────────────────────

pub fn extrair_arquivos(
    arquivos: &[PathBuf],
    base: &Path,
    formato: &str,
    incluir_arvore: bool,
    progress: Option<Arc<Mutex<(usize, usize, String)>>>,
) -> (String, usize) {
    let total = arquivos.len();
    let is_md = formato == "md";
    let mut partes = Vec::new();
    let mut total_linhas = 0usize;

    let nome_projeto = base
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| base.to_string_lossy().to_string());

    // Cabeçalho (será atualizado depois com contagem de linhas)
    let header_start = partes.len();
    if is_md {
        partes.push(format!("# 📦 Contexto do Projeto: {}\n\n", nome_projeto));
        partes.push(format!("> Extraído de `{}`\n", base.display()));
        partes.push(format!("> Total de arquivos: {}\n", total));
        partes.push("LINHAS: [CALCULANDO...]\n\n".to_string());
    } else {
        partes.push(format!("PROJETO: {}\n", nome_projeto));
        partes.push(format!("CAMINHO: {}\n", base.display()));
        partes.push(format!("TOTAL DE ARQUIVOS: {}\n", total));
        partes.push("TOTAL DE LINHAS: [CALCULANDO...]\n\n".to_string());
    }

    // Árvore do projeto
    if incluir_arvore {
        let arvore = gerar_arvore_texto(arquivos, base);
        if is_md {
            partes.push("## 🗂️ Estrutura do Projeto\n\n".to_string());
            partes.push("```\n".to_string());
            partes.push(arvore);
            partes.push("\n```\n\n".to_string());
            partes.push("---\n\n".to_string());
        } else {
            partes.push(format!("{}\n", "=".repeat(60)));
            partes.push("ESTRUTURA DO PROJETO\n".to_string());
            partes.push(format!("{}\n\n", "=".repeat(60)));
            partes.push(arvore);
            partes.push(format!("\n\n{}\n\n", "=".repeat(60)));
        }
    }

    // Arquivos
    for (i, caminho) in arquivos.iter().enumerate() {
        let caminho_relativo = caminho
            .strip_prefix(base)
            .unwrap_or(caminho)
            .to_string_lossy()
            .to_string();

        if let Some(ref prog) = progress {
            if let Ok(mut p) = prog.lock() {
                *p = (i + 1, total, caminho_relativo.clone());
            }
        }

        match std::fs::read_to_string(caminho) {
            Ok(conteudo) => {
                let num_linhas = conteudo.lines().count();
                total_linhas += num_linhas;

                if is_md {
                    let ext = caminho
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
                        .unwrap_or_default();
                    let lang = extensao_linguagem(&ext);
                    partes.push(format!("## 📄 `{}`\n\n", caminho_relativo));
                    partes.push(format!("```{}\n", lang));
                    partes.push(conteudo.clone());
                    if !conteudo.ends_with('\n') {
                        partes.push("\n".to_string());
                    }
                    partes.push("```\n\n".to_string());
                } else {
                    partes.push(format!("\n{}\n", "=".repeat(60)));
                    partes.push(format!("ARQUIVO: {}\n", caminho_relativo));
                    partes.push(format!("{}\n\n", "=".repeat(60)));
                    partes.push(conteudo);
                    partes.push("\n".to_string());
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("invalid utf-8")
                    || err_str.contains("stream did not contain valid UTF-8")
                {
                    if is_md {
                        partes.push(format!(
                            "## 📄 `{}` ⚠️ BINÁRIO IGNORADO\n\n",
                            caminho_relativo
                        ));
                    } else {
                        partes.push(format!("\n{}\n", "=".repeat(60)));
                        partes.push(format!(
                            "ARQUIVO: {} [BINÁRIO IGNORADO]\n",
                            caminho_relativo
                        ));
                        partes.push(format!("{}\n\n", "=".repeat(60)));
                    }
                } else {
                    partes.push(format!(
                        "\n[ERRO AO LER {}]: {}\n",
                        caminho_relativo, e
                    ));
                }
            }
        }
    }

    // Atualizar cabeçalho com contagem de linhas
    let line_info = if is_md {
        format!("> Total de linhas: {}\n\n", total_linhas)
    } else {
        format!("TOTAL DE LINHAS: {}\n\n", total_linhas)
    };
    partes[header_start + 3] = line_info;

    (partes.join(""), total_linhas)
}

// ── Minificar conteúdo para LLM ─────────────────────────────

pub fn minificar(conteudo: &str) -> String {
    let mut resultado = String::with_capacity(conteudo.len());
    let mut linha_vazia_anterior = false;

    for linha in conteudo.lines() {
        let trimmed = linha.trim();

        // Pula linhas totalmente vazias consecutivas
        if trimmed.is_empty() {
            if !linha_vazia_anterior {
                resultado.push('\n');
                linha_vazia_anterior = true;
            }
            continue;
        }
        linha_vazia_anterior = false;

        // Pula linhas decorativas (só ====, ----, etc.)
        if trimmed
            .chars()
            .all(|c| c == '=' || c == '-' || c == '─' || c == '━')
            && trimmed.len() > 3
        {
            continue;
        }

        // Remove indentação excessiva: mantém apenas 1 espaço por nível de tab/4 espaços
        let leading_spaces = linha.len() - linha.trim_start().len();
        let indent_level = leading_spaces / 4;
        if indent_level > 0 {
            for _ in 0..indent_level {
                resultado.push(' ');
            }
        }

        resultado.push_str(trimmed);
        resultado.push('\n');
    }

    resultado
}

