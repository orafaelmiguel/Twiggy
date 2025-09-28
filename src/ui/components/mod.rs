pub mod error_dialog;
pub mod log_viewer;

#[allow(unused_imports)]
pub use error_dialog::*;

#[allow(dead_code)]
pub struct CommitGraph {
    pub zoom_level: f32,
    pub scroll_offset: (f32, f32),
}

#[allow(dead_code)]
impl CommitGraph {
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            scroll_offset: (0.0, 0.0),
        }
    }

    pub fn render(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("Commit Graph Component");
    }
}

#[allow(dead_code)]
pub struct DiffViewer {
    pub show_line_numbers: bool,
    pub syntax_highlighting: bool,
}

#[allow(dead_code)]
impl DiffViewer {
    pub fn new() -> Self {
        Self {
            show_line_numbers: true,
            syntax_highlighting: true,
        }
    }

    pub fn render(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("Diff Viewer Component");
    }
}

#[allow(dead_code)]
pub struct FileTree {
    pub expanded_folders: Vec<String>,
    pub selected_file: Option<String>,
}

#[allow(dead_code)]
impl FileTree {
    pub fn new() -> Self {
        Self {
            expanded_folders: Vec::new(),
            selected_file: None,
        }
    }

    pub fn render(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("File Tree Component");
    }
}

#[allow(dead_code)]
pub struct StatusBar {
    pub current_branch: String,
    pub uncommitted_changes: usize,
}

#[allow(dead_code)]
impl StatusBar {
    pub fn new() -> Self {
        Self {
            current_branch: "main".to_string(),
            uncommitted_changes: 0,
        }
    }

    pub fn render(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Branch: {}", self.current_branch));
            ui.separator();
            ui.label(format!("Changes: {}", self.uncommitted_changes));
        });
    }
}