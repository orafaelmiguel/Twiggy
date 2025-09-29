use crate::git::types::{Commit, CommitId};
use eframe::egui;

pub struct CommitListComponent {
    selected_commit: Option<CommitId>,
    scroll_offset: f32,
    hover_commit: Option<CommitId>,
    item_height: f32,
    visible_range: (usize, usize),
}

impl Default for CommitListComponent {
    fn default() -> Self {
        Self {
            selected_commit: None,
            scroll_offset: 0.0,
            hover_commit: None,
            item_height: 60.0,
            visible_range: (0, 0),
        }
    }
}

impl CommitListComponent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ui: &mut egui::Ui, commits: &[Commit]) -> CommitListResponse {
        let mut response = CommitListResponse {
            selected: self.selected_commit,
            clicked: None,
            double_clicked: None,
        };

        if commits.is_empty() {
            self.render_empty_state(ui);
            return response;
        }

        let available_rect = ui.available_rect_before_wrap();
        let _visible_items = (available_rect.height() / self.item_height).ceil() as usize + 2;
        
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(available_rect.height())
            .show_rows(ui, self.item_height, commits.len(), |ui, row_range| {
                self.visible_range = (row_range.start, row_range.end);
                
                for index in row_range {
                    if index >= commits.len() {
                        break;
                    }
                    
                    let commit = &commits[index];
                    let is_selected = self.selected_commit == Some(commit.id);
                    let is_hovered = self.hover_commit == Some(commit.id);
                    let is_even = index % 2 == 0;

                    let item_response = self.render_commit_item(
                        ui,
                        commit,
                        is_selected,
                        is_hovered,
                        is_even,
                        index,
                    );

                    if item_response.hovered() {
                        self.hover_commit = Some(commit.id);
                    } else if self.hover_commit == Some(commit.id) {
                        self.hover_commit = None;
                    }

                    if item_response.clicked() {
                        self.selected_commit = Some(commit.id);
                        response.clicked = Some(commit.id);
                    }

                    if item_response.double_clicked() {
                        response.double_clicked = Some(commit.id);
                    }
                }
            });

        response
    }

    fn render_empty_state(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("No commits to display");
                ui.add_space(10.0);
                ui.label("Open a repository to view commit history");
                ui.add_space(50.0);
            });
        });
    }

    fn render_commit_item(
        &self,
        ui: &mut egui::Ui,
        commit: &Commit,
        is_selected: bool,
        is_hovered: bool,
        is_even: bool,
        _index: usize,
    ) -> egui::Response {
        let bg_color = if is_selected {
            ui.visuals().selection.bg_fill
        } else if is_hovered {
            ui.visuals().widgets.hovered.bg_fill
        } else if is_even {
            ui.visuals().faint_bg_color
        } else {
            ui.visuals().extreme_bg_color
        };

        let text_color = if is_selected {
            ui.visuals().selection.stroke.color
        } else {
            ui.visuals().text_color()
        };

        let frame = egui::Frame::none()
            .fill(bg_color)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .stroke(if is_selected {
                egui::Stroke::new(1.0, ui.visuals().selection.stroke.color)
            } else {
                egui::Stroke::NONE
            });

        let response = frame.show(ui, |ui| {
            ui.set_min_height(self.item_height - 16.0);
            
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(commit.id.short())
                                .monospace()
                                .color(if is_selected { text_color } else { ui.visuals().weak_text_color() })
                                .size(12.0)
                        )
                    );
                    
                    ui.add_space(8.0);
                    
                    if commit.parents.len() > 1 {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("⚡")
                                    .color(egui::Color32::YELLOW)
                                    .size(14.0)
                            )
                        );
                        ui.add_space(4.0);
                    }
                    
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&commit.summary)
                                .color(text_color)
                                .size(14.0)
                        )
                        .wrap(false)
                        .truncate(true)
                    );
                });
                
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&commit.author.name)
                                .color(if is_selected { text_color } else { ui.visuals().weak_text_color() })
                                .size(11.0)
                        )
                    );
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(
                                    commit.author.time.format("%Y-%m-%d %H:%M").to_string()
                                )
                                .color(if is_selected { text_color } else { ui.visuals().weak_text_color() })
                                .size(11.0)
                                .monospace()
                            )
                        );
                        
                        if commit.parents.len() > 1 {
                            ui.add_space(8.0);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!("{} parents", commit.parents.len()))
                                        .color(if is_selected { text_color } else { ui.visuals().weak_text_color() })
                                        .size(10.0)
                                        .italics()
                                )
                            );
                        }
                    });
                });
            });
        }).response;

        response.interact(egui::Sense::click())
    }

    pub fn handle_keyboard(&mut self, ctx: &egui::Context, commits: &[Commit]) -> bool {
        if commits.is_empty() {
            return false;
        }

        let mut selection_changed = false;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowDown) {
                self.select_next(commits);
                selection_changed = true;
            }

            if i.key_pressed(egui::Key::ArrowUp) {
                self.select_previous(commits);
                selection_changed = true;
            }

            if i.key_pressed(egui::Key::Home) {
                self.select_first(commits);
                selection_changed = true;
            }

            if i.key_pressed(egui::Key::End) {
                self.select_last(commits);
                selection_changed = true;
            }

            if i.key_pressed(egui::Key::PageDown) {
                self.select_page_down(commits);
                selection_changed = true;
            }

            if i.key_pressed(egui::Key::PageUp) {
                self.select_page_up(commits);
                selection_changed = true;
            }
        });

        selection_changed
    }

    fn select_next(&mut self, commits: &[Commit]) {
        if let Some(current_id) = self.selected_commit {
            if let Some(pos) = commits.iter().position(|c| c.id == current_id) {
                if pos + 1 < commits.len() {
                    self.selected_commit = Some(commits[pos + 1].id);
                }
            }
        } else if !commits.is_empty() {
            self.selected_commit = Some(commits[0].id);
        }
    }

    fn select_previous(&mut self, commits: &[Commit]) {
        if let Some(current_id) = self.selected_commit {
            if let Some(pos) = commits.iter().position(|c| c.id == current_id) {
                if pos > 0 {
                    self.selected_commit = Some(commits[pos - 1].id);
                }
            }
        } else if !commits.is_empty() {
            self.selected_commit = Some(commits[commits.len() - 1].id);
        }
    }

    fn select_first(&mut self, commits: &[Commit]) {
        if !commits.is_empty() {
            self.selected_commit = Some(commits[0].id);
        }
    }

    fn select_last(&mut self, commits: &[Commit]) {
        if !commits.is_empty() {
            self.selected_commit = Some(commits[commits.len() - 1].id);
        }
    }

    fn select_page_down(&mut self, commits: &[Commit]) {
        let page_size = 10;
        if let Some(current_id) = self.selected_commit {
            if let Some(pos) = commits.iter().position(|c| c.id == current_id) {
                let new_pos = (pos + page_size).min(commits.len() - 1);
                self.selected_commit = Some(commits[new_pos].id);
            }
        } else if !commits.is_empty() {
            self.selected_commit = Some(commits[0].id);
        }
    }

    fn select_page_up(&mut self, commits: &[Commit]) {
        let page_size = 10;
        if let Some(current_id) = self.selected_commit {
            if let Some(pos) = commits.iter().position(|c| c.id == current_id) {
                let new_pos = pos.saturating_sub(page_size);
                self.selected_commit = Some(commits[new_pos].id);
            }
        } else if !commits.is_empty() {
            self.selected_commit = Some(commits[commits.len() - 1].id);
        }
    }

    pub fn selected_commit(&self) -> Option<CommitId> {
        self.selected_commit
    }

    pub fn set_selected_commit(&mut self, commit_id: Option<CommitId>) {
        self.selected_commit = commit_id;
    }

    pub fn clear_selection(&mut self) {
        self.selected_commit = None;
        self.hover_commit = None;
    }

    pub fn get_visible_range(&self) -> (usize, usize) {
        self.visible_range
    }
}

#[derive(Debug, Clone)]
pub struct CommitListResponse {
    pub selected: Option<CommitId>,
    pub clicked: Option<CommitId>,
    pub double_clicked: Option<CommitId>,
}

impl CommitListResponse {
    pub fn has_selection_changed(&self, previous_selection: Option<CommitId>) -> bool {
        self.selected != previous_selection
    }

    pub fn was_clicked(&self) -> bool {
        self.clicked.is_some()
    }

    pub fn was_double_clicked(&self) -> bool {
        self.double_clicked.is_some()
    }
}