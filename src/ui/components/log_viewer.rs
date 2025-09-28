use eframe::egui;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use crate::error::{Result, TwiggyError};

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(LogLevel::Trace),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            _ => None,
        }
    }

    fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Trace => egui::Color32::GRAY,
            LogLevel::Debug => egui::Color32::LIGHT_BLUE,
            LogLevel::Info => egui::Color32::WHITE,
            LogLevel::Warn => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub raw_line: String,
}

impl LogEntry {
    fn parse(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(5, ' ').collect();
        if parts.len() < 5 {
            return None;
        }

        let timestamp_str = format!("{} {}", parts[0], parts[1]);
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .ok()?
            .with_timezone(&Local);

        let level = LogLevel::from_str(parts[2])?;
        let target = parts[3].to_string();
        let message = parts[4..].join(" ");

        Some(LogEntry {
            timestamp,
            level,
            target,
            message,
            raw_line: line.to_string(),
        })
    }
}

#[derive(Default)]
pub struct LogViewerState {
    pub search_text: String,
    pub selected_levels: Vec<LogLevel>,
    pub auto_scroll: bool,
    pub show_timestamps: bool,
    pub show_targets: bool,
}

pub struct LogViewer {
    entries: Vec<LogEntry>,
    filtered_entries: Vec<usize>,
    state: LogViewerState,
    log_file_path: Option<PathBuf>,
    last_modified: Option<std::time::SystemTime>,
}

impl Default for LogViewer {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            state: LogViewerState {
                selected_levels: vec![LogLevel::Info, LogLevel::Warn, LogLevel::Error],
                auto_scroll: true,
                show_timestamps: true,
                show_targets: true,
                ..Default::default()
            },
            log_file_path: None,
            last_modified: None,
        }
    }
}

impl LogViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_log_file(&mut self, path: PathBuf) -> Result<()> {
        self.log_file_path = Some(path);
        self.reload_logs()
    }

    pub fn reload_logs(&mut self) -> Result<()> {
        if let Some(ref path) = self.log_file_path {
            if path.exists() {
                let metadata = fs::metadata(path)
                    .map_err(|e| TwiggyError::Io { 
                        operation: format!("Failed to read log file metadata: {}", e),
                        source: e.into(),
                    })?;

                let modified = metadata.modified()
                    .map_err(|e| TwiggyError::Io { 
                        operation: format!("Failed to get file modification time: {}", e),
                        source: e.into(),
                    })?;

                if self.last_modified.map_or(true, |last| modified > last) {
                    let content = fs::read_to_string(path)
                    .map_err(|e| TwiggyError::Io { 
                        operation: format!("Failed to read log file: {}", e),
                        source: e.into(),
                    })?;

                    self.entries.clear();
                    for line in content.lines() {
                        if let Some(entry) = LogEntry::parse(line) {
                            self.entries.push(entry);
                        }
                    }

                    self.last_modified = Some(modified);
                    self.apply_filters();
                }
            }
        }
        Ok(())
    }

    fn apply_filters(&mut self) {
        self.filtered_entries.clear();
        
        for (index, entry) in self.entries.iter().enumerate() {
            if !self.state.selected_levels.contains(&entry.level) {
                continue;
            }

            if !self.state.search_text.is_empty() {
                let search_lower = self.state.search_text.to_lowercase();
                if !entry.message.to_lowercase().contains(&search_lower) &&
                   !entry.target.to_lowercase().contains(&search_lower) {
                    continue;
                }
            }

            self.filtered_entries.push(index);
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> Result<()> {
        self.reload_logs()?;

        ui.horizontal(|ui| {
            ui.label("Search:");
            let search_response = ui.text_edit_singleline(&mut self.state.search_text);
            if search_response.changed() {
                self.apply_filters();
            }

            ui.separator();

            ui.label("Levels:");
            let mut levels_changed = false;
            
            for level in [LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug, LogLevel::Trace] {
                let mut selected = self.state.selected_levels.contains(&level);
                if ui.checkbox(&mut selected, format!("{:?}", level)).changed() {
                    if selected {
                        if !self.state.selected_levels.contains(&level) {
                            self.state.selected_levels.push(level);
                        }
                    } else {
                        self.state.selected_levels.retain(|l| l != &level);
                    }
                    levels_changed = true;
                }
            }

            if levels_changed {
                self.apply_filters();
            }

            ui.separator();

            ui.checkbox(&mut self.state.show_timestamps, "Timestamps");
            ui.checkbox(&mut self.state.show_targets, "Targets");
            ui.checkbox(&mut self.state.auto_scroll, "Auto-scroll");

            if ui.button("Clear").clicked() {
                self.entries.clear();
                self.filtered_entries.clear();
            }

            if ui.button("Export").clicked() {
                if let Err(e) = self.export_logs() {
                    tracing::error!("Failed to export logs: {}", e);
                }
            }
        });

        ui.separator();

        let scroll_area = egui::ScrollArea::vertical()
            .stick_to_bottom(self.state.auto_scroll)
            .max_height(ui.available_height());

        scroll_area.show(ui, |ui| {
            egui::Grid::new("log_entries")
                .num_columns(if self.state.show_timestamps && self.state.show_targets { 4 } else if self.state.show_timestamps || self.state.show_targets { 3 } else { 2 })
                .striped(true)
                .show(ui, |ui| {
                    for &entry_index in &self.filtered_entries {
                        if let Some(entry) = self.entries.get(entry_index) {
                            ui.colored_label(entry.level.color(), format!("{:?}", entry.level));

                            if self.state.show_timestamps {
                                ui.label(entry.timestamp.format("%H:%M:%S%.3f").to_string());
                            }

                            if self.state.show_targets {
                                ui.label(&entry.target);
                            }

                            ui.label(&entry.message);
                            ui.end_row();
                        }
                    }
                });
        });

        Ok(())
    }

    fn export_logs(&self) -> Result<()> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("Log files", &["log"])
            .set_file_name("twiggy_logs.txt")
            .save_file()
        {
            let mut content = String::new();
            for &entry_index in &self.filtered_entries {
                if let Some(entry) = self.entries.get(entry_index) {
                    content.push_str(&entry.raw_line);
                    content.push('\n');
                }
            }

            fs::write(&path, content)
                .map_err(|e| TwiggyError::Io { 
                    operation: format!("Failed to export logs: {}", e),
                    source: e.into(),
                })?;

            tracing::info!("Logs exported to: {}", path.display());
        }
        Ok(())
    }
}