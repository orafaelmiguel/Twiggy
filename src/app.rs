use eframe::egui;
use crate::{config::{AppConfig, ThemeType}, error::{Result, TwiggyError}, log_error};
use std::{time::Instant, path::PathBuf};

#[derive(Debug)]
pub struct ErrorState {
    pub message: String,
    pub error_code: u32,
    pub is_recoverable: bool,
    pub suggested_action: Option<String>,
    pub timestamp: Instant,
    pub show_dialog: bool,
}

impl ErrorState {
    pub fn new(error: &TwiggyError) -> Self {
        Self {
            message: error.user_message(),
            error_code: error.error_code(),
            is_recoverable: error.is_recoverable(),
            suggested_action: error.suggested_action(),
            timestamp: Instant::now(),
            show_dialog: true,
        }
    }
}

pub struct TwiggyApp {
    config: AppConfig,
    error_state: Option<ErrorState>,
    notifications: Vec<Notification>,
    last_config_save: Option<Instant>,
    performance_metrics: PerformanceMetrics,
    show_settings: bool,
    settings_tab: SettingsTab,
    temp_config: AppConfig,
}

#[derive(Debug)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: Instant,
    pub auto_dismiss_seconds: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    Window,
    Theme,
    Git,
    Performance,
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub frame_count: u64,
    pub last_frame_time: Option<Instant>,
    pub average_frame_time_ms: f32,
}

impl Default for TwiggyApp {
    fn default() -> Self {
        let config = AppConfig::default();
        Self {
            temp_config: config.clone(),
            config,
            error_state: None,
            notifications: Vec::new(),
            last_config_save: None,
            performance_metrics: PerformanceMetrics::default(),
            show_settings: false,
            settings_tab: SettingsTab::Window,
        }
    }
}

impl TwiggyApp {
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing Twiggy application");
        
        let mut config = match AppConfig::load() {
            Ok(mut cfg) => {
                if let Err(e) = cfg.migrate_if_needed() {
                    tracing::warn!("Configuration migration failed: {}", e);
                }
                cfg
            }
            Err(e) => {
                tracing::warn!("Failed to load configuration: {}. Using defaults.", e);
                AppConfig::default()
            }
        };

        if let Err(e) = config.validate() {
            tracing::warn!("Configuration validation failed: {}. Resetting to defaults.", e);
            config = AppConfig::default();
            if let Err(save_err) = config.save() {
                tracing::error!("Failed to save default configuration: {}", save_err);
            }
        }

        let mut app = Self {
            temp_config: config.clone(),
            config,
            error_state: None,
            notifications: Vec::new(),
            last_config_save: Some(Instant::now()),
            performance_metrics: PerformanceMetrics::default(),
            show_settings: false,
            settings_tab: SettingsTab::Window,
        };

        app.add_notification(
            "Twiggy initialized successfully".to_string(),
            NotificationType::Success,
            Some(3),
        );

        tracing::info!("Twiggy application initialized successfully");
        Ok(app)
    }

    pub fn handle_error(&mut self, error: TwiggyError) {
        log_error!(error);
        
        let error_state = ErrorState::new(&error);
        let is_recoverable = error_state.is_recoverable;
        
        self.error_state = Some(error_state);
        
        if !is_recoverable {
            self.add_notification(
                "Critical error occurred. Application may be unstable.".to_string(),
                NotificationType::Error,
                None,
            );
        }
    }

    pub fn add_notification(&mut self, message: String, notification_type: NotificationType, auto_dismiss_seconds: Option<u32>) {
        let notification = Notification {
            message,
            notification_type,
            timestamp: Instant::now(),
            auto_dismiss_seconds,
        };
        
        self.notifications.push(notification);
        
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    pub fn try_recover_from_error(&mut self) -> Result<()> {
        if let Some(ref error_state) = self.error_state {
            if error_state.is_recoverable {
                match error_state.error_code {
                    3000..=3999 => {
                        tracing::info!("Attempting to recover from configuration error");
                        self.config = AppConfig::default();
                        if let Err(e) = self.config.save() {
                            return Err(e);
                        }
                        self.add_notification(
                            "Configuration reset to defaults".to_string(),
                            NotificationType::Info,
                            Some(5),
                        );
                    }
                    _ => {
                        tracing::info!("Generic error recovery attempted");
                        self.add_notification(
                            "Error recovery attempted".to_string(),
                            NotificationType::Info,
                            Some(3),
                        );
                    }
                }
                
                self.error_state = None;
                return Ok(());
            }
        }
        
        Err(TwiggyError::Application {
            message: "No recoverable error to handle".to_string(),
        })
    }

    fn update_performance_metrics(&mut self) {
        let now = Instant::now();
        
        if let Some(last_frame) = self.performance_metrics.last_frame_time {
            let frame_time = now.duration_since(last_frame).as_millis() as f32;
            
            if self.performance_metrics.frame_count == 0 {
                self.performance_metrics.average_frame_time_ms = frame_time;
            } else {
                let alpha = 0.1;
                self.performance_metrics.average_frame_time_ms = 
                    (1.0 - alpha) * self.performance_metrics.average_frame_time_ms + alpha * frame_time;
            }
        }
        
        self.performance_metrics.last_frame_time = Some(now);
        self.performance_metrics.frame_count += 1;
    }

    fn auto_save_config_if_needed(&mut self) {
        if let Some(last_save) = self.last_config_save {
            if last_save.elapsed().as_secs() > 300 {
                if let Err(e) = self.config.save() {
                    self.handle_error(e);
                } else {
                    self.last_config_save = Some(Instant::now());
                    tracing::debug!("Configuration auto-saved");
                }
            }
        }
    }

    fn cleanup_old_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|notification| {
            if let Some(auto_dismiss) = notification.auto_dismiss_seconds {
                now.duration_since(notification.timestamp).as_secs() < auto_dismiss as u64
            } else {
                true
            }
        });
    }

    fn render_error_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut should_recover = false;
        
        if let Some(ref error_state) = self.error_state {
            if error_state.show_dialog {
                egui::Window::new("⚠️ Error")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.set_min_width(400.0);
                        
                        ui.vertical(|ui| {
                            ui.add_space(10.0);
                            
                            ui.label(egui::RichText::new(&error_state.message)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(220, 50, 50)));
                            
                            ui.add_space(10.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("Error Code:");
                                ui.label(format!("{}", error_state.error_code));
                            });
                            
                            if let Some(ref action) = error_state.suggested_action {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("Suggested Action:")
                                    .strong());
                                ui.label(action);
                            }
                            
                            ui.add_space(15.0);
                            
                            ui.horizontal(|ui| {
                                if error_state.is_recoverable && ui.button("🔄 Try Recovery").clicked() {
                                    should_recover = true;
                                }
                                
                                if ui.button("✅ OK").clicked() {
                                    should_close = true;
                                }
                            });
                        });
                    });
            }
        }
        
        if should_close {
            if let Some(ref mut error_state) = self.error_state {
                error_state.show_dialog = false;
            }
        }
        
        if should_recover {
            if let Err(e) = self.try_recover_from_error() {
                tracing::error!("Recovery failed: {}", e);
            }
        }
    }

    fn render_notifications(&mut self, ctx: &egui::Context) {
        if self.notifications.is_empty() {
            return;
        }

        let mut to_remove = Vec::new();
        
        egui::Area::new("notifications")
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for (index, notification) in self.notifications.iter().enumerate() {
                        let frame = egui::Frame::default()
                            .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 40, 240))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)))
                            .rounding(5.0)
                            .inner_margin(egui::Margin::same(10.0));
                        
                        frame.show(ui, |ui| {
                            ui.set_max_width(300.0);
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&notification.message)
                                    .size(14.0)
                                    .color(match notification.notification_type {
                                        NotificationType::Info => egui::Color32::from_rgb(70, 130, 180),
                                        NotificationType::Warning => egui::Color32::from_rgb(255, 165, 0),
                                        NotificationType::Error => egui::Color32::from_rgb(220, 50, 50),
                                        NotificationType::Success => egui::Color32::from_rgb(50, 180, 50),
                                    }));
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("✕").clicked() {
                                        to_remove.push(index);
                                    }
                                });
                            });
                        });
                        
                        ui.add_space(5.0);
                    }
                });
            });
            
        for &index in to_remove.iter().rev() {
            self.notifications.remove(index);
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }

        egui::Window::new("⚙️ Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Window, "🪟 Window");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Theme, "🎨 Theme");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Git, "📦 Git");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Performance, "⚡ Performance");
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.settings_tab {
                        SettingsTab::Window => self.render_window_settings(ui),
                        SettingsTab::Theme => self.render_theme_settings(ui),
                        SettingsTab::Git => self.render_git_settings(ui),
                        SettingsTab::Performance => self.render_performance_settings(ui),
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        self.config = self.temp_config.clone();
                        if let Err(e) = self.config.save() {
                            self.handle_error(e);
                        } else {
                            self.add_notification(
                                "Settings saved successfully".to_string(),
                                NotificationType::Success,
                                Some(3),
                            );
                        }
                        self.show_settings = false;
                    }

                    if ui.button("🔄 Reset to Defaults").clicked() {
                        self.temp_config = AppConfig::default();
                    }

                    if ui.button("❌ Cancel").clicked() {
                        self.temp_config = self.config.clone();
                        self.show_settings = false;
                    }
                });
            });
    }

    fn render_window_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Window Settings");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut self.temp_config.window.width)
                .clamp_range(400.0..=4000.0)
                .suffix(" px"));
        });

        ui.horizontal(|ui| {
            ui.label("Height:");
            ui.add(egui::DragValue::new(&mut self.temp_config.window.height)
                .clamp_range(300.0..=3000.0)
                .suffix(" px"));
        });

        ui.checkbox(&mut self.temp_config.window.maximized, "Start maximized");
        ui.checkbox(&mut self.temp_config.window.remember_position, "Remember window position");

        if let (Some(x), Some(y)) = (self.temp_config.window.position_x, self.temp_config.window.position_y) {
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.label(format!("({:.0}, {:.0})", x, y));
            });
        }
    }

    fn render_theme_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Theme Type:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.temp_config.theme.theme_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.temp_config.theme.theme_type, ThemeType::Light, "Light");
                    ui.selectable_value(&mut self.temp_config.theme.theme_type, ThemeType::Dark, "Dark");
                    ui.selectable_value(&mut self.temp_config.theme.theme_type, ThemeType::System, "System");
                });
        });
        
        ui.horizontal(|ui| {
            ui.label("Font Size:");
            ui.add(egui::DragValue::new(&mut self.temp_config.theme.font_size)
                .clamp_range(8.0..=32.0)
                .suffix(" px"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Dark Mode:");
            ui.checkbox(&mut self.temp_config.theme.dark_mode, "");
        });
        
        ui.horizontal(|ui| {
            ui.label("Accent Color:");
            ui.text_edit_singleline(&mut self.temp_config.theme.accent_color);
        });
    }

    fn render_git_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Default Clone Path:");
            if let Some(ref mut path) = self.temp_config.git.default_clone_path {
                ui.text_edit_singleline(&mut path.to_string_lossy().to_string());
            } else {
                let mut path_str = String::new();
                if ui.text_edit_singleline(&mut path_str).changed() && !path_str.is_empty() {
                    self.temp_config.git.default_clone_path = Some(PathBuf::from(path_str));
                }
            }
            if ui.button("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.temp_config.git.default_clone_path = Some(path);
                }
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Max Commits:");
            ui.add(egui::DragValue::new(&mut self.temp_config.git.max_commits)
                .clamp_range(1..=10000));
        });
        
        ui.horizontal(|ui| {
            ui.label("Default Branch:");
            ui.text_edit_singleline(&mut self.temp_config.git.default_branch);
        });
        
        ui.horizontal(|ui| {
            ui.label("Auto Fetch:");
            ui.checkbox(&mut self.temp_config.git.auto_fetch, "");
        });
        
        ui.horizontal(|ui| {
            ui.label("Fetch Interval:");
            ui.add(egui::DragValue::new(&mut self.temp_config.git.fetch_interval_minutes)
                .clamp_range(1..=1440)
                .suffix(" min"));
        });
    }

    fn render_performance_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance Settings");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Enable Caching:");
            ui.checkbox(&mut self.temp_config.performance.enable_caching, "");
        });
        
        ui.horizontal(|ui| {
            ui.label("Cache Size:");
            ui.add(egui::DragValue::new(&mut self.temp_config.performance.cache_size_mb)
                .clamp_range(1..=2048)
                .suffix(" MB"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Background Operations:");
            ui.checkbox(&mut self.temp_config.performance.background_operations, "");
        });
    }
}

impl eframe::App for TwiggyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_performance_metrics();
        self.auto_save_config_if_needed();
        self.cleanup_old_notifications();
        
        self.render_error_dialog(ctx);
        self.render_notifications(ctx);
        self.render_settings_dialog(ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("🌿 Twiggy");
                ui.label("Lightning-fast Git Visualization Tool");
                ui.add_space(20.0);
                
                ui.label("Phase 5: Production Error Handling - ✅ Active");
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Frame Time:");
                    ui.label(format!("{:.1}ms", self.performance_metrics.average_frame_time_ms));
                    
                    ui.separator();
                    
                    ui.label("Frames:");
                    ui.label(format!("{}", self.performance_metrics.frame_count));
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("🧪 Test Error").clicked() {
                        let test_error = TwiggyError::Application {
                            message: "This is a test error for demonstration".to_string(),
                        };
                        self.handle_error(test_error);
                    }
                    
                    if ui.button("📢 Test Notification").clicked() {
                        self.add_notification(
                            "This is a test notification".to_string(),
                            NotificationType::Info,
                            Some(5),
                        );
                    }
                    
                    if ui.button("⚙️ Settings").clicked() {
                        self.temp_config = self.config.clone();
                        self.show_settings = true;
                    }
                    
                    if ui.button("🔄 Reset Config").clicked() {
                        if let Err(e) = self.config.reset_to_defaults() {
                            self.handle_error(e);
                        } else {
                            self.add_notification(
                                "Configuration reset successfully".to_string(),
                                NotificationType::Success,
                                Some(3),
                            );
                        }
                    }
                });
            });
        });
    }
}