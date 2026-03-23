// ══════════════════════════════════════════════════════════════
//  Graphical interface (egui / eframe)
// ══════════════════════════════════════════════════════════════

use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::{
    default_ignored_dirs, extensions_from_languages, default_ignored_extensions, LANGUAGES,
};
use crate::extractor::{build_tree, extract_files, minify, FileNode};

// ── Auxiliary types ────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
pub enum OutputFormat {
    Markdown,
    Text,
}

pub enum ExtractionState {
    Idle,
    Running(Arc<Mutex<(usize, usize, String)>>),
    Done,
}

// ── App ─────────────────────────────────────────────────────

pub struct App {
    // State
    project_path: String,
    tree_nodes: Vec<FileNode>,
    output_format: OutputFormat,
    include_tree: bool,

    // Ignored folders/extensions
    ignored_dirs: Vec<String>,
    ignored_exts: Vec<String>,
    new_dir: String,
    new_ext: String,
    ext_filter: String,

    // Search
    search: String,

    // Language filter
    filter_by_language: bool,
    selected_languages: Vec<bool>,

    // Result
    result_content: String,
    result_label: String,
    extraction_state: ExtractionState,
    extraction_result: Arc<Mutex<Option<(String, usize)>>>,

    // Preview
    preview_content: String,
    preview_label: String,
    show_preview: bool,

    // Feedback
    status_label: String,
    copied_feedback: Option<std::time::Instant>,
    copied_mini_feedback: Option<std::time::Instant>,

    // Count
    total_found: usize,

    // Theme
    dark_mode: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            project_path: String::new(),
            tree_nodes: Vec::new(),
            output_format: OutputFormat::Markdown,
            include_tree: true,
            ignored_dirs: default_ignored_dirs(),
            ignored_exts: default_ignored_extensions(),
            new_dir: String::new(),
            new_ext: String::new(),
            ext_filter: String::new(),
            search: String::new(),
            filter_by_language: false,
            selected_languages: vec![false; LANGUAGES.len()],
            result_content: String::new(),
            result_label: "Click a file for preview or extract to see the result here."
                .to_string(),
            extraction_state: ExtractionState::Idle,
            extraction_result: Arc::new(Mutex::new(None)),
            preview_content: String::new(),
            preview_label: String::new(),
            show_preview: false,
            status_label: "Select a folder to start.".to_string(),
            copied_feedback: None,
            copied_mini_feedback: None,
            total_found: 0,
            dark_mode: true,
        }
    }

    fn load_tree(&mut self) {
        let path = PathBuf::from(shellexpand(&self.project_path));
        if !path.is_dir() {
            self.status_label =
                format!("Error: '{}' is not a valid folder.", self.project_path);
            return;
        }
        self.project_path = path.to_string_lossy().to_string();

        let dirs_set: HashSet<String> = self.ignored_dirs.iter().cloned().collect();
        let exts_set: HashSet<String> = self.ignored_exts.iter().cloned().collect();

        let filter = if self.filter_by_language {
            let lang_exts = extensions_from_languages(&self.selected_languages);
            if lang_exts.is_empty() {
                None
            } else {
                Some(lang_exts)
            }
        } else if self.ext_filter.trim().is_empty() {
            None
        } else {
            Some(
                self.ext_filter
                    .split_whitespace()
                    .map(String::from)
                    .collect::<HashSet<String>>(),
            )
        };

        self.tree_nodes = build_tree(&path, &dirs_set, &exts_set, &filter).unwrap_or_default();

        self.total_found = self.tree_nodes.iter().map(|n| n.total_files()).sum();
        self.result_content.clear();
        self.preview_content.clear();
        self.show_preview = false;
        self.result_label =
            "Click a file for preview or extract to see the result here.".to_string();
        self.status_label = format!(
            "{} file(s) found. Check/uncheck and click Extract.",
            self.total_found
        );
    }

    fn count_selected(&self) -> usize {
        self.tree_nodes.iter().map(|n| n.file_count()).sum()
    }

    fn extract(&mut self) {
        if self.project_path.is_empty() {
            self.status_label = "Select a project folder first!".to_string();
            return;
        }

        let files: Vec<PathBuf> = self
            .tree_nodes
            .iter()
            .flat_map(|n| n.collect_checked_files())
            .collect();

        if files.is_empty() {
            self.status_label = "No files selected!".to_string();
            return;
        }

        let base = PathBuf::from(&self.project_path);
        let format = match self.output_format {
            OutputFormat::Markdown => "md",
            OutputFormat::Text => "txt",
        };
        let include_tree = self.include_tree;
        let progress = Arc::new(Mutex::new((0usize, files.len(), String::new())));
        let result_holder = self.extraction_result.clone();
        let progress_clone = progress.clone();

        self.extraction_state = ExtractionState::Running(progress);
        self.result_label = "Extracting…".to_string();
        self.show_preview = false;

        *result_holder.lock().unwrap() = None;

        let format_owned = format.to_string();
        thread::spawn(move || {
            let result = extract_files(
                &files,
                &base,
                &format_owned,
                include_tree,
                Some(progress_clone),
            );
            *result_holder.lock().unwrap() = Some(result);
        });
    }

    fn show_preview(&mut self, path: &PathBuf, base: &str) {
        if !self.result_content.is_empty() {
            return;
        }
        let relative_path = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        match std::fs::read_to_string(path) {
            Ok(mut content) => {
                let max_preview = 50_000;
                let mut truncated = false;
                if content.len() > max_preview {
                    content.truncate(max_preview);
                    truncated = true;
                }
                if truncated {
                    content.push_str(&format!(
                        "\n\n... [TRUNCATED - {} characters] ...",
                        max_preview
                    ));
                }
                self.preview_label = format!("Preview: {}", relative_path);
                self.preview_content = content;
                self.show_preview = true;
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("utf-8") || err_str.contains("UTF-8") {
                    self.preview_label = format!("{} - Binary file", relative_path);
                } else {
                    self.preview_label = format!("Error reading {}: {}", relative_path, e);
                }
                self.preview_content.clear();
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
        // Check if extraction finished
        if let ExtractionState::Running(ref progress) = self.extraction_state {
            let (current, total, name) = progress.lock().unwrap().clone();
            if current > 0 {
                self.status_label = format!("Extracting ({}/{}): {}", current, total, name);
            }
            ctx.request_repaint();

            if let Some((result, lines)) = self.extraction_result.lock().unwrap().take() {
                let total = self.count_selected();
                self.result_content = result;
                self.result_label = format!(
                    "Extraction result — {} file(s), {} lines — Ready to copy!",
                    total, lines
                );
                self.status_label = format!(
                    "{} file(s) extracted — {} lines of code — Use 'Copy All' to copy.",
                    total, lines
                );
                self.show_preview = false;
                self.extraction_state = ExtractionState::Done;
            }
        }

        // Reset copy feedback
        if let Some(instant) = self.copied_feedback {
            if instant.elapsed().as_secs() >= 2 {
                self.copied_feedback = None;
            }
        }
        if let Some(instant) = self.copied_mini_feedback {
            if instant.elapsed().as_secs() >= 2 {
                self.copied_mini_feedback = None;
            }
        }

        // ── Apply theme ──
        if self.dark_mode {
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::from_rgb(212, 212, 212));
            ctx.set_visuals(visuals);
        } else {
            let mut visuals = egui::Visuals::light();
            visuals.override_text_color = Some(egui::Color32::from_rgb(30, 30, 30));
            ctx.set_visuals(visuals);
        }

        // ── Top bar ──
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Project folder:");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.project_path)
                        .desired_width(400.0)
                        .hint_text("Paste or type the path…"),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.load_tree();
                }
                if ui.button("Load").clicked() {
                    self.load_tree();
                }
                if ui.button("Choose folder…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.project_path = path.to_string_lossy().to_string();
                        self.load_tree();
                    }
                }
                if ui.button("Reload").clicked() {
                    self.load_tree();
                }

                // Theme toggle (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_label = if self.dark_mode { "☀ Light" } else { "🌙 Dark" };
                    if ui.button(theme_label).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });
            ui.add_space(4.0);
        });

        // ── Status bar (bottom) ──
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

        // ── Right panel: configuration ──
        egui::SidePanel::right("config_panel")
            .default_width(280.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Ignored folders
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Ignored Folders").strong());
                        ui.add_space(4.0);

                        let mut to_remove = None;
                        egui::ScrollArea::vertical()
                            .id_salt("dirs_list")
                            .max_height(120.0)
                            .show(ui, |ui| {
                                for (i, d) in self.ignored_dirs.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.monospace(d);
                                        if ui.small_button("✕").clicked() {
                                            to_remove = Some(i);
                                        }
                                    });
                                }
                            });
                        if let Some(i) = to_remove {
                            self.ignored_dirs.remove(i);
                        }

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_dir)
                                    .desired_width(150.0)
                                    .hint_text("new folder…"),
                            );
                            if ui.button("+").clicked() {
                                let val = self.new_dir.trim().to_string();
                                if !val.is_empty() && !self.ignored_dirs.contains(&val) {
                                    self.ignored_dirs.push(val);
                                    self.new_dir.clear();
                                }
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Ignored extensions
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Ignored Extensions").strong());
                        ui.add_space(4.0);

                        let mut to_remove = None;
                        egui::ScrollArea::vertical()
                            .id_salt("exts_list")
                            .max_height(120.0)
                            .show(ui, |ui| {
                                for (i, e) in self.ignored_exts.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.monospace(e);
                                        if ui.small_button("✕").clicked() {
                                            to_remove = Some(i);
                                        }
                                    });
                                }
                            });
                        if let Some(i) = to_remove {
                            self.ignored_exts.remove(i);
                        }

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_ext)
                                    .desired_width(150.0)
                                    .hint_text(".ext"),
                            );
                            if ui.button("+").clicked() {
                                let mut val = self.new_ext.trim().to_string();
                                if !val.starts_with('.') {
                                    val = format!(".{}", val);
                                }
                                if !val.is_empty() && !self.ignored_exts.contains(&val) {
                                    self.ignored_exts.push(val);
                                    self.new_ext.clear();
                                }
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Extension filter
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Filter by Extension").strong());
                        ui.label("Ex: .py .ts .js (empty = all)");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.ext_filter)
                                .desired_width(ui.available_width())
                                .hint_text(".py .ts .js"),
                        );
                    });

                    ui.add_space(6.0);

                    // Language filter
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Filter by Language").strong());
                        let changed = ui
                            .checkbox(
                                &mut self.filter_by_language,
                                "Enable language filter",
                            )
                            .changed();

                        if self.filter_by_language {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.small_button("All").clicked() {
                                    for sel in self.selected_languages.iter_mut() {
                                        *sel = true;
                                    }
                                }
                                if ui.small_button("None").clicked() {
                                    for sel in self.selected_languages.iter_mut() {
                                        *sel = false;
                                    }
                                }
                            });
                            ui.add_space(2.0);

                            egui::ScrollArea::vertical()
                                .id_salt("lang_list")
                                .max_height(180.0)
                                .show(ui, |ui| {
                                    for (i, lang) in LANGUAGES.iter().enumerate() {
                                        ui.checkbox(
                                            &mut self.selected_languages[i],
                                            format!("{} {}", lang.name, lang.name),
                                        );
                                    }
                                });

                            let exts = extensions_from_languages(&self.selected_languages);
                            if !exts.is_empty() {
                                ui.add_space(2.0);
                                let mut sorted: Vec<_> = exts.iter().collect();
                                sorted.sort();
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Extensions: {}",
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

                        if changed && !self.project_path.is_empty() {
                            // placeholder for reload
                        }
                    });

                    ui.add_space(6.0);

                    // Output options
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Output Options").strong());
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut self.output_format,
                                OutputFormat::Markdown,
                                "Markdown (.md)",
                            );
                            ui.radio_value(
                                &mut self.output_format,
                                OutputFormat::Text,
                                "Text (.txt)",
                            );
                        });
                        ui.checkbox(&mut self.include_tree, "Include project tree");
                    });

                    ui.add_space(8.0);

                    // Extract button
                    let btn = egui::Button::new(
                        egui::RichText::new("Extract Selected Files").strong(),
                    )
                    .min_size(egui::vec2(ui.available_width(), 36.0));
                    if ui.add(btn).clicked() {
                        self.extract();
                    }
                });
            });

        // ── Central panel ──
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();

            ui.horizontal_top(|ui| {
                // ━━━ LEFT PANEL: tree ━━━
                ui.vertical(|ui| {
                    ui.set_min_width(available.x * 0.4);
                    ui.set_max_width(available.x * 0.5);

                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Project Files").strong());

                        // Toolbar
                        ui.horizontal(|ui| {
                            if ui.button("All").clicked() {
                                for node in &mut self.tree_nodes {
                                    node.set_checked(true);
                                }
                            }
                            if ui.button("None").clicked() {
                                for node in &mut self.tree_nodes {
                                    node.set_checked(false);
                                }
                            }
                            if ui.button("Invert").clicked() {
                                for node in &mut self.tree_nodes {
                                    node.invert_files();
                                }
                            }

                            let sel_count = self.count_selected();
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(format!("{} selected", sel_count));
                                },
                            );
                        });

                        // Search
                        ui.horizontal(|ui| {
                            ui.label("Search:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search)
                                    .desired_width(ui.available_width())
                                    .hint_text("Search file…"),
                            );
                        });

                        ui.add_space(4.0);

                        // File tree
                        let search_term = self.search.trim().to_lowercase();
                        let base = self.project_path.clone();

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
                            self.show_preview(&path, &base);
                        }
                    });
                });

                ui.add_space(4.0);

                // ━━━ RIGHT PANEL: result / preview ━━━
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Extraction Result").strong());

                        // Result toolbar
                        ui.horizontal(|ui| {
                            if !self.result_content.is_empty() {
                                ui.label(&self.result_label);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Save
                                        if ui.button("Save").clicked() {
                                            let ext = match self.output_format {
                                                OutputFormat::Markdown => "md",
                                                OutputFormat::Text => "txt",
                                            };
                                            let file = rfd::FileDialog::new()
                                                .set_file_name(&format!(
                                                    "full_code.{}",
                                                    ext
                                                ))
                                                .add_filter("Markdown", &["md"])
                                                .add_filter("Text", &["txt"])
                                                .add_filter("All", &["*"])
                                                .save_file();
                                            if let Some(path) = file {
                                                if let Err(e) = std::fs::write(
                                                    &path,
                                                    &self.result_content,
                                                ) {
                                                    self.status_label =
                                                        format!("Error saving: {}", e);
                                                } else {
                                                    self.status_label = format!(
                                                        "Saved to: {}",
                                                        path.display()
                                                    );
                                                }
                                            }
                                        }

                                        // Copy minified
                                        let btn_mini_text =
                                            if self.copied_mini_feedback.is_some() {
                                                "Minified copied!"
                                            } else {
                                                "Copy Minified"
                                            };
                                        if ui
                                            .button(btn_mini_text)
                                            .on_hover_text(
                                                "Copies minified content (no extra spaces) for pasting in LLMs",
                                            )
                                            .clicked()
                                        {
                                            let mini = minify(&self.result_content);
                                            let original_len = self.result_content.len();
                                            let mini_len = mini.len();
                                            let savings = if original_len > 0 {
                                                100.0
                                                    - (mini_len as f64 / original_len as f64
                                                        * 100.0)
                                            } else {
                                                0.0
                                            };
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                let _ = clipboard.set_text(&mini);
                                                self.copied_mini_feedback =
                                                    Some(std::time::Instant::now());
                                                self.status_label = format!(
                                                    "Minified copied! {} → {} chars ({:.0}% smaller)",
                                                    original_len, mini_len, savings
                                                );
                                            }
                                        }

                                        // Copy all
                                        let btn_text = if self.copied_feedback.is_some() {
                                            "Copied!"
                                        } else {
                                            "Copy All"
                                        };
                                        if ui.button(btn_text).clicked() {
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                let _ = clipboard
                                                    .set_text(&self.result_content);
                                                self.copied_feedback =
                                                    Some(std::time::Instant::now());
                                                self.status_label =
                                                    "Content copied to clipboard!".to_string();
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

                        // Text area
                        let text_to_show = if !self.result_content.is_empty() {
                            &self.result_content
                        } else if self.show_preview {
                            &self.preview_content
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

// ── Tree rendering ─────────────────────────────────────────

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

