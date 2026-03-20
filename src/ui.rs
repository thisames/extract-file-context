// ══════════════════════════════════════════════════════════════
//  Interface gráfica (egui / eframe)
// ══════════════════════════════════════════════════════════════

use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::{
    dirs_ignorados_padrao, extensoes_das_linguagens, extensoes_ignoradas_padrao, LINGUAGENS,
};
use crate::extractor::{build_tree, extrair_arquivos, minificar, FileNode};

// ── Tipos auxiliares ────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
pub enum FormatoSaida {
    Markdown,
    Texto,
}

pub enum ExtractionState {
    Idle,
    Running(Arc<Mutex<(usize, usize, String)>>),
    Done,
}

// ── App ─────────────────────────────────────────────────────

pub struct App {
    // Estado
    caminho_projeto: String,
    tree_nodes: Vec<FileNode>,
    formato_saida: FormatoSaida,
    incluir_arvore: bool,

    // Pastas/extensões ignoradas
    dirs_ignorados: Vec<String>,
    exts_ignoradas: Vec<String>,
    novo_dir: String,
    nova_ext: String,
    filtro_ext: String,

    // Busca
    busca: String,

    // Filtro por linguagem
    filtrar_por_linguagem: bool,
    linguagens_selecionadas: Vec<bool>,

    // Resultado
    resultado_conteudo: String,
    result_label: String,
    extraction_state: ExtractionState,
    extraction_result: Arc<Mutex<Option<(String, usize)>>>,

    // Preview
    preview_conteudo: String,
    preview_label: String,
    show_preview: bool,

    // Feedback
    status_label: String,
    copiado_feedback: Option<std::time::Instant>,
    copiado_mini_feedback: Option<std::time::Instant>,

    // Contagem
    total_encontrados: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            caminho_projeto: String::new(),
            tree_nodes: Vec::new(),
            formato_saida: FormatoSaida::Markdown,
            incluir_arvore: true,
            dirs_ignorados: dirs_ignorados_padrao(),
            exts_ignoradas: extensoes_ignoradas_padrao(),
            novo_dir: String::new(),
            nova_ext: String::new(),
            filtro_ext: String::new(),
            busca: String::new(),
            filtrar_por_linguagem: false,
            linguagens_selecionadas: vec![false; LINGUAGENS.len()],
            resultado_conteudo: String::new(),
            result_label: "Clique em um arquivo para preview ou extraia para ver o resultado aqui."
                .to_string(),
            extraction_state: ExtractionState::Idle,
            extraction_result: Arc::new(Mutex::new(None)),
            preview_conteudo: String::new(),
            preview_label: String::new(),
            show_preview: false,
            status_label: "Selecione uma pasta para começar.".to_string(),
            copiado_feedback: None,
            copiado_mini_feedback: None,
            total_encontrados: 0,
        }
    }

    fn carregar_arvore(&mut self) {
        let path = PathBuf::from(shellexpand(&self.caminho_projeto));
        if !path.is_dir() {
            self.status_label =
                format!("Erro: '{}' não é uma pasta válida.", self.caminho_projeto);
            return;
        }
        self.caminho_projeto = path.to_string_lossy().to_string();

        let dirs_set: HashSet<String> = self.dirs_ignorados.iter().cloned().collect();
        let exts_set: HashSet<String> = self.exts_ignoradas.iter().cloned().collect();

        let filtro = if self.filtrar_por_linguagem {
            let lang_exts = extensoes_das_linguagens(&self.linguagens_selecionadas);
            if lang_exts.is_empty() {
                None
            } else {
                Some(lang_exts)
            }
        } else if self.filtro_ext.trim().is_empty() {
            None
        } else {
            Some(
                self.filtro_ext
                    .split_whitespace()
                    .map(String::from)
                    .collect::<HashSet<String>>(),
            )
        };

        self.tree_nodes = build_tree(&path, &dirs_set, &exts_set, &filtro).unwrap_or_default();

        self.total_encontrados = self.tree_nodes.iter().map(|n| n.total_files()).sum();
        self.resultado_conteudo.clear();
        self.preview_conteudo.clear();
        self.show_preview = false;
        self.result_label =
            "Clique em um arquivo para preview ou extraia para ver o resultado aqui.".to_string();
        self.status_label = format!(
            "{} arquivo(s) encontrado(s). Marque/desmarque e clique em Extrair.",
            self.total_encontrados
        );
    }

    fn contar_selecionados(&self) -> usize {
        self.tree_nodes.iter().map(|n| n.file_count()).sum()
    }

    fn extrair(&mut self) {
        if self.caminho_projeto.is_empty() {
            self.status_label = "Selecione uma pasta do projeto primeiro!".to_string();
            return;
        }

        let arquivos: Vec<PathBuf> = self
            .tree_nodes
            .iter()
            .flat_map(|n| n.collect_checked_files())
            .collect();

        if arquivos.is_empty() {
            self.status_label = "Nenhum arquivo selecionado!".to_string();
            return;
        }

        let base = PathBuf::from(&self.caminho_projeto);
        let formato = match self.formato_saida {
            FormatoSaida::Markdown => "md",
            FormatoSaida::Texto => "txt",
        };
        let incluir_arvore = self.incluir_arvore;
        let progress = Arc::new(Mutex::new((0usize, arquivos.len(), String::new())));
        let result_holder = self.extraction_result.clone();
        let progress_clone = progress.clone();

        self.extraction_state = ExtractionState::Running(progress);
        self.result_label = "⏳ Extraindo…".to_string();
        self.show_preview = false;

        *result_holder.lock().unwrap() = None;

        let formato_owned = formato.to_string();
        thread::spawn(move || {
            let resultado = extrair_arquivos(
                &arquivos,
                &base,
                &formato_owned,
                incluir_arvore,
                Some(progress_clone),
            );
            *result_holder.lock().unwrap() = Some(resultado);
        });
    }

    fn mostrar_preview(&mut self, path: &PathBuf, base: &str) {
        if !self.resultado_conteudo.is_empty() {
            return;
        }
        let caminho_relativo = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        match std::fs::read_to_string(path) {
            Ok(mut conteudo) => {
                let max_preview = 50_000;
                let mut truncado = false;
                if conteudo.len() > max_preview {
                    conteudo.truncate(max_preview);
                    truncado = true;
                }
                if truncado {
                    conteudo.push_str(&format!(
                        "\n\n... [TRUNCADO - {} caracteres] ...",
                        max_preview
                    ));
                }
                self.preview_label = format!("👁️ Preview: {}", caminho_relativo);
                self.preview_conteudo = conteudo;
                self.show_preview = true;
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("utf-8") || err_str.contains("UTF-8") {
                    self.preview_label = format!("⚠️ {} — Arquivo binário", caminho_relativo);
                } else {
                    self.preview_label = format!("❌ Erro ao ler {}: {}", caminho_relativo, e);
                }
                self.preview_conteudo.clear();
                self.show_preview = true;
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────

/// Simples expansão de ~ no início do caminho
fn shellexpand(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return format!("{}{}", home.to_string_lossy(), &s[1..]);
        }
    }
    s.to_string()
}

// ── eframe::App ─────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Verifica se a extração terminou
        if let ExtractionState::Running(ref progress) = self.extraction_state {
            let (current, total, name) = progress.lock().unwrap().clone();
            if current > 0 {
                self.status_label = format!("Extraindo ({}/{}): {}", current, total, name);
            }
            ctx.request_repaint();

            if let Some((result, linhas)) = self.extraction_result.lock().unwrap().take() {
                let total = self.contar_selecionados();
                self.resultado_conteudo = result;
                self.result_label = format!(
                    "✅ Resultado da extração — {} arquivo(s), {} linhas — Pronto para copiar!",
                    total, linhas
                );
                self.status_label = format!(
                    "✅ {} arquivo(s) extraído(s) — {} linhas de código — Use '📋 Copiar tudo' para copiar.",
                    total, linhas
                );
                self.show_preview = false;
                self.extraction_state = ExtractionState::Done;
            }
        }

        // Reset feedback de cópia
        if let Some(instant) = self.copiado_feedback {
            if instant.elapsed().as_secs() >= 2 {
                self.copiado_feedback = None;
            }
        }
        if let Some(instant) = self.copiado_mini_feedback {
            if instant.elapsed().as_secs() >= 2 {
                self.copiado_mini_feedback = None;
            }
        }

        // ── Barra superior ──
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Pasta do projeto:");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.caminho_projeto)
                        .desired_width(400.0)
                        .hint_text("Cole ou digite o caminho…"),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.carregar_arvore();
                }
                if ui.button("▶ Carregar").clicked() {
                    self.carregar_arvore();
                }
                if ui.button("📂 Escolher pasta…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.caminho_projeto = path.to_string_lossy().to_string();
                        self.carregar_arvore();
                    }
                }
                if ui.button("🔄 Recarregar").clicked() {
                    self.carregar_arvore();
                }
            });
            ui.add_space(4.0);
        });

        // ── Barra de status (inferior) ──
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(&self.status_label);
                if let ExtractionState::Running(ref progress) = self.extraction_state {
                    let (current, total, _) = progress.lock().unwrap().clone();
                    if total > 0 {
                        let frac = current as f32 / total as f32;
                        ui.add(
                            egui::ProgressBar::new(frac)
                                .desired_width(200.0)
                                .text(format!("{}/{}", current, total)),
                        );
                    }
                }
            });
            ui.add_space(2.0);
        });

        // ── Painel direito: configurações ──
        egui::SidePanel::right("config_panel")
            .default_width(280.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Pastas ignoradas
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("📁 Pastas ignoradas").strong());
                        ui.add_space(4.0);

                        let mut to_remove = None;
                        egui::ScrollArea::vertical()
                            .id_salt("dirs_list")
                            .max_height(120.0)
                            .show(ui, |ui| {
                                for (i, d) in self.dirs_ignorados.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.monospace(d);
                                        if ui.small_button("✕").clicked() {
                                            to_remove = Some(i);
                                        }
                                    });
                                }
                            });
                        if let Some(i) = to_remove {
                            self.dirs_ignorados.remove(i);
                        }

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.novo_dir)
                                    .desired_width(150.0)
                                    .hint_text("nova pasta…"),
                            );
                            if ui.button("+").clicked() {
                                let val = self.novo_dir.trim().to_string();
                                if !val.is_empty() && !self.dirs_ignorados.contains(&val) {
                                    self.dirs_ignorados.push(val);
                                    self.novo_dir.clear();
                                }
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Extensões ignoradas
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🚫 Extensões ignoradas").strong());
                        ui.add_space(4.0);

                        let mut to_remove = None;
                        egui::ScrollArea::vertical()
                            .id_salt("exts_list")
                            .max_height(120.0)
                            .show(ui, |ui| {
                                for (i, e) in self.exts_ignoradas.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.monospace(e);
                                        if ui.small_button("✕").clicked() {
                                            to_remove = Some(i);
                                        }
                                    });
                                }
                            });
                        if let Some(i) = to_remove {
                            self.exts_ignoradas.remove(i);
                        }

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.nova_ext)
                                    .desired_width(150.0)
                                    .hint_text(".ext"),
                            );
                            if ui.button("+").clicked() {
                                let mut val = self.nova_ext.trim().to_string();
                                if !val.starts_with('.') {
                                    val = format!(".{}", val);
                                }
                                if !val.is_empty() && !self.exts_ignoradas.contains(&val) {
                                    self.exts_ignoradas.push(val);
                                    self.nova_ext.clear();
                                }
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Filtro por extensão
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🔍 Filtrar por extensão").strong());
                        ui.label("Ex: .py .ts .js (vazio = todas)");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.filtro_ext)
                                .desired_width(ui.available_width())
                                .hint_text(".py .ts .js"),
                        );
                    });

                    ui.add_space(6.0);

                    // Filtro por linguagem
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🌐 Filtrar por linguagem").strong());
                        let changed = ui
                            .checkbox(
                                &mut self.filtrar_por_linguagem,
                                "Ativar filtro por linguagem",
                            )
                            .changed();

                        if self.filtrar_por_linguagem {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.small_button("Todas").clicked() {
                                    for sel in self.linguagens_selecionadas.iter_mut() {
                                        *sel = true;
                                    }
                                }
                                if ui.small_button("Nenhuma").clicked() {
                                    for sel in self.linguagens_selecionadas.iter_mut() {
                                        *sel = false;
                                    }
                                }
                            });
                            ui.add_space(2.0);

                            egui::ScrollArea::vertical()
                                .id_salt("lang_list")
                                .max_height(180.0)
                                .show(ui, |ui| {
                                    for (i, lang) in LINGUAGENS.iter().enumerate() {
                                        ui.checkbox(
                                            &mut self.linguagens_selecionadas[i],
                                            format!("{} {}", lang.emoji, lang.nome),
                                        );
                                    }
                                });

                            let exts = extensoes_das_linguagens(&self.linguagens_selecionadas);
                            if !exts.is_empty() {
                                ui.add_space(2.0);
                                let mut sorted: Vec<_> = exts.iter().collect();
                                sorted.sort();
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Extensões: {}",
                                        sorted
                                            .iter()
                                            .map(|s| s.as_str())
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                    ))
                                    .small()
                                    .weak(),
                                );
                            }
                        }

                        if changed && !self.caminho_projeto.is_empty() {
                            // placeholder para recarregar
                        }
                    });

                    ui.add_space(6.0);

                    // Opções de saída
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("⚙️ Opções de saída").strong());
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut self.formato_saida,
                                FormatoSaida::Markdown,
                                "Markdown (.md)",
                            );
                            ui.radio_value(
                                &mut self.formato_saida,
                                FormatoSaida::Texto,
                                "Texto (.txt)",
                            );
                        });
                        ui.checkbox(&mut self.incluir_arvore, "Incluir árvore do projeto");
                    });

                    ui.add_space(8.0);

                    // Botão de extração
                    let btn = egui::Button::new(
                        egui::RichText::new("🚀 Extrair arquivos selecionados").strong(),
                    )
                    .min_size(egui::vec2(ui.available_width(), 36.0));
                    if ui.add(btn).clicked() {
                        self.extrair();
                    }
                });
            });

        // ── Painel central ──
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();

            ui.horizontal_top(|ui| {
                // ━━━ PAINEL ESQUERDO: árvore ━━━
                ui.vertical(|ui| {
                    ui.set_min_width(available.x * 0.4);
                    ui.set_max_width(available.x * 0.5);

                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Arquivos do projeto").strong());

                        // Toolbar
                        ui.horizontal(|ui| {
                            if ui.button("✅ Tudo").clicked() {
                                for node in &mut self.tree_nodes {
                                    node.set_checked(true);
                                }
                            }
                            if ui.button("❌ Nada").clicked() {
                                for node in &mut self.tree_nodes {
                                    node.set_checked(false);
                                }
                            }
                            if ui.button("🔄 Inverter").clicked() {
                                for node in &mut self.tree_nodes {
                                    node.invert_files();
                                }
                            }

                            let sel_count = self.contar_selecionados();
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(format!("{} selecionado(s)", sel_count));
                                },
                            );
                        });

                        // Busca
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.busca)
                                    .desired_width(ui.available_width())
                                    .hint_text("Buscar arquivo…"),
                            );
                        });

                        ui.add_space(4.0);

                        // Árvore de arquivos
                        let search_term = self.busca.trim().to_lowercase();
                        let base = self.caminho_projeto.clone();

                        let mut preview_path: Option<PathBuf> = None;

                        egui::ScrollArea::both()
                            .id_salt("file_tree")
                            .max_height(available.y - 100.0)
                            .show(ui, |ui| {
                                let nodes = &mut self.tree_nodes;
                                for node in nodes.iter_mut() {
                                    render_tree_node(ui, node, &search_term, &mut preview_path);
                                }
                            });

                        if let Some(path) = preview_path {
                            self.mostrar_preview(&path, &base);
                        }
                    });
                });

                ui.add_space(4.0);

                // ━━━ PAINEL DIREITO: resultado / preview ━━━
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Resultado da extração").strong());

                        // Toolbar do resultado
                        ui.horizontal(|ui| {
                            if !self.resultado_conteudo.is_empty() {
                                ui.label(&self.result_label);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Salvar
                                        if ui.button("💾 Salvar").clicked() {
                                            let ext = match self.formato_saida {
                                                FormatoSaida::Markdown => "md",
                                                FormatoSaida::Texto => "txt",
                                            };
                                            let file = rfd::FileDialog::new()
                                                .set_file_name(&format!(
                                                    "codigo_completo.{}",
                                                    ext
                                                ))
                                                .add_filter("Markdown", &["md"])
                                                .add_filter("Texto", &["txt"])
                                                .add_filter("Todos", &["*"])
                                                .save_file();
                                            if let Some(path) = file {
                                                if let Err(e) = std::fs::write(
                                                    &path,
                                                    &self.resultado_conteudo,
                                                ) {
                                                    self.status_label =
                                                        format!("Erro ao salvar: {}", e);
                                                } else {
                                                    self.status_label = format!(
                                                        "💾 Salvo em: {}",
                                                        path.display()
                                                    );
                                                }
                                            }
                                        }

                                        // Copiar minificado
                                        let btn_mini_text =
                                            if self.copiado_mini_feedback.is_some() {
                                                "✅ Mini copiado!"
                                            } else {
                                                "🗜️ Copiar mini"
                                            };
                                        if ui
                                            .button(btn_mini_text)
                                            .on_hover_text(
                                                "Copia o conteúdo minificado (sem espaços extras) para colar em LLMs",
                                            )
                                            .clicked()
                                        {
                                            let mini = minificar(&self.resultado_conteudo);
                                            let original_len = self.resultado_conteudo.len();
                                            let mini_len = mini.len();
                                            let economia = if original_len > 0 {
                                                100.0
                                                    - (mini_len as f64 / original_len as f64
                                                        * 100.0)
                                            } else {
                                                0.0
                                            };
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                let _ = clipboard.set_text(&mini);
                                                self.copiado_mini_feedback =
                                                    Some(std::time::Instant::now());
                                                self.status_label = format!(
                                                    "🗜️ Minificado copiado! {} → {} chars ({:.0}% menor)",
                                                    original_len, mini_len, economia
                                                );
                                            }
                                        }

                                        // Copiar
                                        let btn_text = if self.copiado_feedback.is_some() {
                                            "✅ Copiado!"
                                        } else {
                                            "📋 Copiar tudo"
                                        };
                                        if ui.button(btn_text).clicked() {
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                let _ = clipboard
                                                    .set_text(&self.resultado_conteudo);
                                                self.copiado_feedback =
                                                    Some(std::time::Instant::now());
                                                self.status_label =
                                                    "📋 Conteúdo copiado para a área de transferência!"
                                                        .to_string();
                                            }
                                        }
                                    },
                                );
                            } else if self.show_preview {
                                ui.label(&self.preview_label);
                            } else {
                                ui.label(&self.result_label);
                            }
                        });

                        ui.add_space(4.0);

                        // Área de texto
                        let text_to_show = if !self.resultado_conteudo.is_empty() {
                            &self.resultado_conteudo
                        } else if self.show_preview {
                            &self.preview_conteudo
                        } else {
                            ""
                        };

                        egui::ScrollArea::both()
                            .id_salt("result_text")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut text_to_show.to_string())
                                        .desired_width(ui.available_width())
                                        .desired_rows(30)
                                        .font(egui::TextStyle::Monospace)
                                        .interactive(false),
                                );
                            });
                    });
                });
            });
        });
    }
}

// ── Renderização da árvore ──────────────────────────────────

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    search_term: &str,
    preview_path: &mut Option<PathBuf>,
) {
    if !search_term.is_empty() && !node.matches_search(search_term) {
        return;
    }

    if node.is_dir {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut node.checked, "").changed() {
                let state = node.checked;
                for child in &mut node.children {
                    child.set_checked(state);
                }
            }
            let header = egui::CollapsingHeader::new(format!("📁 {}", node.name))
                .id_salt(&node.path)
                .default_open(node.expanded || !search_term.is_empty());
            header.show(ui, |ui| {
                for child in &mut node.children {
                    render_tree_node(ui, child, search_term, preview_path);
                }
            });
        });
    } else {
        ui.horizontal(|ui| {
            ui.checkbox(&mut node.checked, "");
            let resp = ui.selectable_label(false, format!("📄 {}", node.name));
            if resp.clicked() {
                *preview_path = Some(node.path.clone());
            }
        });
    }
}

