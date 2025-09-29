use eframe::egui;
use crate::{config::{AppConfig, ThemeType}, error::{Result, TwiggyError}, log_error, logging::{log_performance, log_memory_usage}, ui::components::log_viewer::LogViewer, git::repository::GitRepository};
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
    pending_window_changes: bool,
    last_window_state: Option<WindowState>,
    log_viewer: LogViewer,
    show_log_viewer: bool,
    show_about: bool,
    show_shortcuts: bool,
    current_repository: Option<GitRepository>,
    repository_loading: bool,
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
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
}

impl WindowState {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            width: config.window.width,
            height: config.window.height,
            maximized: config.window.maximized,
            position_x: config.window.position_x,
            position_y: config.window.position_y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    Window,
    Theme,
    Git,
    Performance,
    Logging,
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
            pending_window_changes: false,
            last_window_state: None,
            log_viewer: LogViewer::new(),
            show_log_viewer: false,
            show_about: false,
            show_shortcuts: false,
            current_repository: None,
            repository_loading: false,
        }
    }
}

impl TwiggyApp {
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing Twiggy application");
        
        let mut config = log_performance("config_load", || {
            match AppConfig::load() {
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
            }
        });

        log_memory_usage("config_loaded");

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
            pending_window_changes: false,
            last_window_state: None,
            log_viewer: LogViewer::new(),
            show_log_viewer: false,
            show_about: false,
            show_shortcuts: false,
            current_repository: None,
            repository_loading: false,
        };

        app.add_notification(
            "Twiggy initialized successfully".to_string(),
            NotificationType::Success,
            Some(3),
        );

        log_memory_usage("app_initialized");
        tracing::info!("Twiggy application initialized successfully");
        Ok(app)
    }

    pub fn handle_error(&mut self, error: TwiggyError) {
        tracing::error!("Handling application error: {}", error);
        log_error!(error);
        
        let error_state = ErrorState::new(&error);
        let is_recoverable = error_state.is_recoverable;
        
        tracing::debug!("Error is recoverable: {}", is_recoverable);
        
        self.error_state = Some(error_state);
        
        if !is_recoverable {
            tracing::warn!("Critical error occurred, application may be unstable");
            self.add_notification(
                "Critical error occurred. Application may be unstable.".to_string(),
                NotificationType::Error,
                None,
            );
        }
    }

    pub fn add_notification(&mut self, message: String, notification_type: NotificationType, auto_dismiss_seconds: Option<u32>) {
        tracing::debug!("Adding notification: {:?} - {}", notification_type, message);
        
        let notification = Notification {
            message,
            notification_type,
            timestamp: Instant::now(),
            auto_dismiss_seconds,
        };
        
        self.notifications.push(notification);
        
        if self.notifications.len() > 10 {
            tracing::debug!("Notification queue full, removing oldest notification");
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
            
            if frame_time > 16.0 {
                tracing::debug!("Slow frame detected: {:.2}ms (target: 16ms)", frame_time);
            }
            
            if self.performance_metrics.frame_count % 1000 == 0 {
                tracing::trace!("Performance metrics - Frame: {}, Avg: {:.2}ms", 
                    self.performance_metrics.frame_count, 
                    self.performance_metrics.average_frame_time_ms);
            }
        }
        
        self.performance_metrics.last_frame_time = Some(now);
        self.performance_metrics.frame_count += 1;
    }

    fn auto_save_config_if_needed(&mut self) {
        if let Some(last_save) = self.last_config_save {
            if last_save.elapsed().as_secs() > 300 {
                log_performance("config_auto_save", || {
                    if let Err(e) = self.config.save() {
                        self.handle_error(e);
                    } else {
                        self.last_config_save = Some(Instant::now());
                        tracing::debug!("Configuration auto-saved");
                    }
                });
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

    fn render_settings_dialog(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.show_settings {
            return;
        }

        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Window, "Window");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Theme, "Theme");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Git, "Git");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Performance, "Performance");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Logging, "Logging");
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.settings_tab {
                        SettingsTab::Window => self.render_window_settings(ui, ctx),
                        SettingsTab::Theme => self.render_theme_settings(ui, ctx),
                        SettingsTab::Git => self.render_git_settings(ui, ctx),
                        SettingsTab::Performance => self.render_performance_settings(ui, ctx),
                        SettingsTab::Logging => self.render_logging_settings(ui, ctx),
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        match self.apply_configuration_with_frame(ctx, frame) {
                            Ok(()) => {
                                self.add_notification(
                                    "Settings applied successfully".to_string(),
                                    NotificationType::Success,
                                    Some(3),
                                );
                            }
                            Err(e) => {
                                self.add_notification(
                                    format!("Failed to apply settings: {}", e),
                                    NotificationType::Error,
                                    Some(5),
                                );
                            }
                        }
                    }

                    if ui.button("💾 Save").clicked() {
                        match self.apply_configuration_with_frame(ctx, frame) {
                            Ok(()) => {
                                if let Err(e) = self.config.save() {
                                    self.add_notification(
                                        format!("Failed to save settings: {}", e),
                                        NotificationType::Error,
                                        Some(5),
                                    );
                                } else {
                                    self.add_notification(
                                        "Settings saved successfully".to_string(),
                                        NotificationType::Success,
                                        Some(3),
                                    );
                                    self.show_settings = false;
                                }
                            }
                            Err(e) => {
                                self.add_notification(
                                    format!("Failed to apply settings: {}", e),
                                    NotificationType::Error,
                                    Some(5),
                                );
                            }
                        }
                    }

                    if ui.button("Reset to Defaults").clicked() {
                        self.temp_config = AppConfig::default();
                        self.apply_theme_to_temp_context(ctx);
                        ctx.request_repaint();
                    }

                    if ui.button("Cancel").clicked() {
                        self.temp_config = self.config.clone();
                        self.apply_theme_to_context(ctx);
                        self.show_settings = false;
                    }
                });
            });
    }

    fn render_window_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Window Settings");
        ui.add_space(10.0);

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Width:");
            if ui.add(egui::Slider::new(&mut self.temp_config.window.width, 400.0..=4000.0)
                .suffix(" px")).changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Height:");
            if ui.add(egui::Slider::new(&mut self.temp_config.window.height, 300.0..=3000.0)
                .suffix(" px")).changed() {
                changed = true;
            }
        });

        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            let button_text = if is_maximized { "Restore Window" } else { "Maximize Window" };
            
            if ui.button(button_text).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                changed = true;
            }
            
            ui.label("Toggle between normal and maximized window");
        });

        ui.add_space(10.0);


        if let (Some(x), Some(y)) = (self.temp_config.window.position_x, self.temp_config.window.position_y) {
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.label(format!("({:.0}, {:.0})", x, y));
            });
        }

        if changed {
            ctx.request_repaint();
        }
    }

    fn render_theme_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Theme Settings");
        ui.add_space(10.0);

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Theme Type:");
            let response = egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.temp_config.theme.theme_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.temp_config.theme.theme_type, ThemeType::Light, "Light");
                    ui.selectable_value(&mut self.temp_config.theme.theme_type, ThemeType::Dark, "Dark");
                    ui.selectable_value(&mut self.temp_config.theme.theme_type, ThemeType::System, "System");
                });
            if response.response.changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Font Size:");
            if ui.add(egui::Slider::new(&mut self.temp_config.theme.font_size, 8.0..=32.0)
                .suffix(" px")).changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Dark Mode Override:");
            if ui.checkbox(&mut self.temp_config.theme.dark_mode, "Force dark mode").changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Accent Color:");
            let mut color_text = self.temp_config.theme.accent_color.clone();
            if ui.text_edit_singleline(&mut color_text).changed() {
                self.temp_config.theme.accent_color = color_text;
                changed = true;
            }
            
            if let Ok(color) = self.parse_hex_color(&self.temp_config.theme.accent_color) {
                let mut color32 = color;
                if ui.color_edit_button_srgba(&mut color32).changed() {
                    self.temp_config.theme.accent_color = format!(
                        "#{:02X}{:02X}{:02X}",
                        color32.r(),
                        color32.g(),
                        color32.b()
                    );
                    changed = true;
                }
            }
        });

        if changed {
            self.apply_theme_to_temp_context(ctx);
            ctx.request_repaint();
        }
    }

    fn render_git_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Git Settings");
        ui.add_space(10.0);

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Max Commits:");
            if ui.add(egui::Slider::new(&mut self.temp_config.git.max_commits, 100..=50000)
                .logarithmic(true)).changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Auto Fetch:");
            if ui.checkbox(&mut self.temp_config.git.auto_fetch, "Automatically fetch from remote").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Show Stashes:");
            if ui.checkbox(&mut self.temp_config.git.show_stashes, "Display stashes in history").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Default Clone Path:");
            ui.text_edit_singleline(&mut self.temp_config.git.default_clone_path);
            
            if ui.button("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(&self.temp_config.git.default_clone_path)
                    .pick_folder() {
                    self.temp_config.git.default_clone_path = path.to_string_lossy().to_string();
                    changed = true;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Fetch Interval (minutes):");
            if ui.add(egui::Slider::new(&mut self.temp_config.git.fetch_interval_minutes, 1..=1440)
                .suffix(" min")).changed() {
                changed = true;
            }
        });

        if changed {
            ctx.request_repaint();
        }
    }

    fn render_performance_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Performance Settings");
        ui.add_space(10.0);

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Enable Caching:");
            if ui.checkbox(&mut self.temp_config.performance.enable_caching, "Cache repository data").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Cache Size (MB):");
            if ui.add(egui::Slider::new(&mut self.temp_config.performance.cache_size_mb, 10..=2048)
                .logarithmic(true)
                .suffix(" MB")).changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Background Operations:");
            if ui.checkbox(&mut self.temp_config.performance.enable_background_operations, "Enable background tasks").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Max Background Threads:");
            if ui.add(egui::Slider::new(&mut self.temp_config.performance.max_background_threads, 1..=16)).changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Render FPS Limit:");
            if ui.add(egui::Slider::new(&mut self.temp_config.performance.target_fps, 30..=144)
                .suffix(" FPS")).changed() {
                changed = true;
            }
        });

        if changed {
            ctx.request_repaint();
        }
    }

    fn render_logging_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Logging Settings");
        ui.add_space(10.0);

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Log Level:");
            let current_level = match self.temp_config.logging.level {
                crate::config::LogLevel::Error => 0,
                crate::config::LogLevel::Warn => 1,
                crate::config::LogLevel::Info => 2,
                crate::config::LogLevel::Debug => 3,
                crate::config::LogLevel::Trace => 4,
            };
            let mut selected = current_level;
            
            egui::ComboBox::from_label("")
                .selected_text(match current_level {
                    0 => "Error",
                    1 => "Warn",
                    2 => "Info",
                    3 => "Debug",
                    4 => "Trace",
                    _ => "Info",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, 0, "Error");
                    ui.selectable_value(&mut selected, 1, "Warn");
                    ui.selectable_value(&mut selected, 2, "Info");
                    ui.selectable_value(&mut selected, 3, "Debug");
                    ui.selectable_value(&mut selected, 4, "Trace");
                });
            
            if selected != current_level {
                self.temp_config.logging.level = match selected {
                    0 => crate::config::LogLevel::Error,
                    1 => crate::config::LogLevel::Warn,
                    2 => crate::config::LogLevel::Info,
                    3 => crate::config::LogLevel::Debug,
                    4 => crate::config::LogLevel::Trace,
                    _ => crate::config::LogLevel::Info,
                };
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("File Logging:");
            if ui.checkbox(&mut self.temp_config.logging.file_enabled, "Enable file logging").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Console Logging:");
            if ui.checkbox(&mut self.temp_config.logging.console_enabled, "Enable console logging").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Max File Size (MB):");
            let mut size_mb = (self.temp_config.logging.max_file_size / (1024 * 1024)) as f32;
            if ui.add(egui::Slider::new(&mut size_mb, 1.0..=100.0)
                .suffix(" MB")).changed() {
                self.temp_config.logging.max_file_size = (size_mb * 1024.0 * 1024.0) as u64;
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Max Log Files:");
            if ui.add(egui::Slider::new(&mut self.temp_config.logging.max_files, 1..=20)).changed() {
                changed = true;
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label("Log Directory:");
        if let Some(log_dir) = &self.temp_config.logging.log_directory {
            ui.label(format!("📁 {}", log_dir));
        } else {
            ui.label("📁 Default system directory");
        }

        ui.horizontal(|ui| {
            if ui.button("Open Log Directory").clicked() {
                let log_dir = self.temp_config.logging.log_directory.clone()
                    .or_else(|| {
                        directories::ProjectDirs::from("dev", "twiggy", "Twiggy")
                            .map(|dirs| dirs.data_dir().join("logs").to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|| "logs".to_string());
                
                if let Err(e) = std::process::Command::new("explorer")
                    .arg(&log_dir)
                    .spawn() {
                    self.add_notification(
                        format!("Failed to open log directory: {}", e),
                        NotificationType::Error,
                        Some(5),
                    );
                }
            }

            if ui.button("Clear Logs").clicked() {
                let log_dir = self.temp_config.logging.log_directory.clone()
                    .or_else(|| {
                        directories::ProjectDirs::from("dev", "twiggy", "Twiggy")
                            .map(|dirs| dirs.data_dir().join("logs").to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|| "logs".to_string());
                
                let log_path = std::path::PathBuf::from(&log_dir);
                if log_path.exists() {
                    match std::fs::read_dir(&log_path) {
                        Ok(entries) => {
                            let mut cleared_count = 0;
                            for entry in entries {
                                if let Ok(entry) = entry {
                                    if entry.path().extension().map_or(false, |ext| ext == "log") {
                                        if std::fs::remove_file(entry.path()).is_ok() {
                                            cleared_count += 1;
                                        }
                                    }
                                }
                            }
                            self.add_notification(
                                format!("Cleared {} log files", cleared_count),
                                NotificationType::Success,
                                Some(3),
                            );
                        }
                        Err(e) => {
                            self.add_notification(
                                format!("Failed to clear logs: {}", e),
                                NotificationType::Error,
                                Some(5),
                            );
                        }
                    }
                }
            }

            if ui.button("View Logs").clicked() {
                let log_dir = self.temp_config.logging.log_directory.clone()
                    .or_else(|| {
                        directories::ProjectDirs::from("dev", "twiggy", "Twiggy")
                            .map(|dirs| dirs.data_dir().join("logs").to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|| "logs".to_string());
                
                let log_path = std::path::PathBuf::from(&log_dir);
                if let Ok(entries) = std::fs::read_dir(&log_path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if entry.path().extension().map_or(false, |ext| ext == "log") {
                                if let Err(e) = self.log_viewer.set_log_file(entry.path()) {
                                    self.add_notification(
                                        format!("Failed to load log file: {}", e),
                                        NotificationType::Error,
                                        Some(5),
                                    );
                                } else {
                                    self.show_log_viewer = true;
                                }
                                break;
                            }
                        }
                    }
                } else {
                    self.add_notification(
                        "No log files found".to_string(),
                        NotificationType::Warning,
                        Some(3),
                    );
                }
            }
        });

        if changed {
            ctx.request_repaint();
        }
    }

    fn apply_theme_to_context(&self, ctx: &egui::Context) {
        let mut visuals = match self.config.theme.theme_type {
            ThemeType::Light => egui::Visuals::light(),
            ThemeType::Dark => egui::Visuals::dark(),
            ThemeType::System => {
                if self.config.theme.dark_mode {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                }
            }
        };

        if let Ok(accent_color) = self.parse_hex_color(&self.config.theme.accent_color) {
            visuals.selection.bg_fill = accent_color;
            visuals.hyperlink_color = accent_color;
        }

        ctx.set_visuals(visuals);

        let fonts = egui::FontDefinitions::default();
        
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(self.config.theme.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(self.config.theme.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(self.config.theme.font_size * 1.2, egui::FontFamily::Proportional),
        );
        
        ctx.set_fonts(fonts);
        ctx.set_style(style);
    }

    fn apply_theme_to_temp_context(&self, ctx: &egui::Context) {
        let mut visuals = match self.temp_config.theme.theme_type {
            ThemeType::Light => egui::Visuals::light(),
            ThemeType::Dark => egui::Visuals::dark(),
            ThemeType::System => {
                if self.temp_config.theme.dark_mode {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                }
            }
        };

        if let Ok(accent_color) = self.parse_hex_color(&self.temp_config.theme.accent_color) {
            visuals.selection.bg_fill = accent_color;
            visuals.hyperlink_color = accent_color;
        }

        ctx.set_visuals(visuals);

        let fonts = egui::FontDefinitions::default();
        
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(self.temp_config.theme.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(self.temp_config.theme.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(self.temp_config.theme.font_size * 1.2, egui::FontFamily::Proportional),
        );
        
        ctx.set_fonts(fonts);
        ctx.set_style(style);
    }

    fn apply_configuration(&mut self, ctx: &egui::Context) -> Result<()> {
        tracing::debug!("Applying configuration changes");
        
        if let Err(e) = self.temp_config.validate() {
            tracing::error!("Configuration validation failed: {}", e);
            return Err(TwiggyError::Config {
                message: format!("Configuration validation failed: {}", e),
            });
        }

        let theme_changed = self.config.theme != self.temp_config.theme;
        let window_changed = self.config.window != self.temp_config.window;
        
        self.config = self.temp_config.clone();
        
        if theme_changed {
            tracing::info!("Theme configuration changed, applying new theme");
            self.apply_theme_to_context(ctx);
        }
        
        if window_changed {
            tracing::info!("Window configuration changed");
        }
        
        self.last_config_save = Some(Instant::now());
        tracing::debug!("Configuration applied successfully");
        
        Ok(())
    }

    fn apply_configuration_with_frame(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<()> {
        tracing::debug!("Applying configuration with frame");
        
        if let Err(e) = self.temp_config.validate() {
            tracing::error!("Configuration validation failed: {}", e);
            return Err(TwiggyError::Config {
                message: format!("Configuration validation failed: {}", e),
            });
        }

        let window_changed = self.config.window.width != self.temp_config.window.width
            || self.config.window.height != self.temp_config.window.height
            || self.config.window.maximized != self.temp_config.window.maximized
            || self.config.window.position_x != self.temp_config.window.position_x
            || self.config.window.position_y != self.temp_config.window.position_y;

        let theme_changed = self.config.theme != self.temp_config.theme;

        self.config = self.temp_config.clone();
        
        if window_changed {
            tracing::info!("Window settings changed, applying new window configuration");
            self.apply_window_settings(ctx);
        }
        
        if theme_changed {
            tracing::info!("Theme settings changed, applying new theme");
            self.apply_theme_to_context(ctx);
        }
        
        if let Err(e) = self.config.save() {
            tracing::error!("Failed to save configuration: {}", e);
            return Err(TwiggyError::Config {
                message: format!("Failed to save configuration: {}", e),
            });
        }
        
        tracing::info!("Configuration saved successfully");
        self.last_config_save = Some(Instant::now());
        
        Ok(())
    }

    fn apply_window_settings(&mut self, ctx: &egui::Context) {
        tracing::debug!("Applying window settings");
        
        if let Err(e) = self.validate_window_settings(&self.config) {
            tracing::error!("Window settings validation failed: {}", e);
            self.handle_error(TwiggyError::Config {
                message: format!("Invalid window configuration: {}", e),
            });
            return;
        }
        
        let window_state = WindowState::from_config(&self.config);
        
        if self.config.window.maximized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            
            let size = egui::Vec2::new(self.config.window.width, self.config.window.height);
            if self.is_valid_window_size(size) {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            } else {
                tracing::warn!("Invalid window size detected, using fallback");
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(1200.0, 800.0)));
            }
        }
        
        if self.config.window.remember_position {
            if let (Some(x), Some(y)) = (self.config.window.position_x, self.config.window.position_y) {
                let pos = egui::Pos2::new(x, y);
                if self.is_valid_window_position(pos) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                } else {
                    tracing::warn!("Invalid window position detected, centering window");
                }
            }
        }
        
        self.last_window_state = Some(window_state);
        self.pending_window_changes = false;
    }

    fn apply_window_settings_from_temp(&mut self, ctx: &egui::Context) {
        tracing::debug!("Applying temporary window settings");
        
        if let Err(e) = self.validate_window_settings(&self.temp_config) {
            tracing::warn!("Temporary window settings validation failed: {}", e);
            self.add_notification(
                format!("Invalid window settings: {}", e),
                NotificationType::Warning,
                Some(5),
            );
            return;
        }
        
        if self.temp_config.window.maximized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            
            let size = egui::Vec2::new(self.temp_config.window.width, self.temp_config.window.height);
            if self.is_valid_window_size(size) {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
        }
        
        if self.temp_config.window.remember_position {
            if let (Some(x), Some(y)) = (self.temp_config.window.position_x, self.temp_config.window.position_y) {
                let pos = egui::Pos2::new(x, y);
                if self.is_valid_window_position(pos) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                }
            }
        }
    }

    fn validate_window_settings(&self, config: &AppConfig) -> std::result::Result<(), String> {
        let size = egui::Vec2::new(config.window.width, config.window.height);
        if !self.is_valid_window_size(size) {
            let min_width = if cfg!(target_os = "macos") { 300.0 } else { 400.0 };
            let min_height = if cfg!(target_os = "macos") { 200.0 } else { 300.0 };
            let max_width = if cfg!(target_os = "windows") { 7680.0 } else { 5120.0 };
            let max_height = if cfg!(target_os = "windows") { 4320.0 } else { 2880.0 };
            
            return Err(format!(
                "Window size {}x{} is outside valid range ({}-{} x {}-{})",
                config.window.width, config.window.height,
                min_width, max_width, min_height, max_height
            ));
        }
        
        if let (Some(x), Some(y)) = (config.window.position_x, config.window.position_y) {
            let pos = egui::Pos2::new(x, y);
            if !self.is_valid_window_position(pos) {
                return Err(format!("Window position ({}, {}) is outside platform bounds", x, y));
            }
        }
        
        Ok(())
    }

    fn is_valid_window_size(&self, size: egui::Vec2) -> bool {
        let min_width = if cfg!(target_os = "macos") { 300.0 } else { 400.0 };
        let min_height = if cfg!(target_os = "macos") { 200.0 } else { 300.0 };
        
        let max_width = if cfg!(target_os = "windows") { 7680.0 } else { 5120.0 };
        let max_height = if cfg!(target_os = "windows") { 4320.0 } else { 2880.0 };
        
        size.x >= min_width && size.x <= max_width && size.y >= min_height && size.y <= max_height
    }

    fn is_valid_window_position(&self, pos: egui::Pos2) -> bool {
        let bounds = if cfg!(target_os = "windows") {
            (-1920.0, 7680.0, -1080.0, 4320.0)
        } else if cfg!(target_os = "macos") {
            (-2560.0, 5120.0, -1440.0, 2880.0)
        } else {
            (-1920.0, 5120.0, -1080.0, 2880.0)
        };
        
        pos.x >= bounds.0 && pos.x <= bounds.1 && pos.y >= bounds.2 && pos.y <= bounds.3
    }

    fn get_platform_safe_window_size(&self) -> egui::Vec2 {
        if cfg!(target_os = "macos") {
            egui::Vec2::new(1024.0, 768.0)
        } else if cfg!(target_os = "windows") {
            egui::Vec2::new(1200.0, 800.0)
        } else {
            egui::Vec2::new(1100.0, 750.0)
        }
    }

    fn handle_window_operation_error(&mut self, operation: &str, error: &str) {
        tracing::error!("Window operation '{}' failed: {}", operation, error);
        
        self.add_notification(
            format!("Window operation failed: {}", error),
            NotificationType::Error,
            Some(10),
        );
        
        if operation.contains("resize") || operation.contains("size") {
            tracing::info!("Attempting to recover with platform-safe window size");
            let safe_size = self.get_platform_safe_window_size();
            self.config.window.width = safe_size.x;
            self.config.window.height = safe_size.y;
            self.temp_config.window.width = safe_size.x;
            self.temp_config.window.height = safe_size.y;
        }
        
        if operation.contains("position") {
            tracing::info!("Resetting window position to platform default");
            self.config.window.position_x = None;
            self.config.window.position_y = None;
            self.temp_config.window.position_x = None;
            self.temp_config.window.position_y = None;
        }
        
        if operation.contains("maximized") && cfg!(target_os = "linux") {
            tracing::info!("Maximization failed on Linux, using windowed mode");
            self.config.window.maximized = false;
            self.temp_config.window.maximized = false;
        }
    }

    pub fn apply_initial_window_config(&mut self, ctx: &egui::Context) {
        log_performance("apply_initial_window_config", || {
            tracing::info!("Applying initial window configuration");
            
            if let Err(e) = self.validate_window_settings(&self.config) {
                tracing::warn!("Initial window configuration invalid, using platform defaults: {}", e);
                let safe_size = self.get_platform_safe_window_size();
                self.config.window.width = safe_size.x;
                self.config.window.height = safe_size.y;
                self.config.window.maximized = false;
                self.config.window.position_x = None;
                self.config.window.position_y = None;
            }
            
            let window_state = WindowState::from_config(&self.config);
            
            if self.config.window.maximized {
                if cfg!(target_os = "linux") {
                    tracing::debug!("Applying maximization on Linux with delay");
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.get_platform_safe_window_size()));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                }
            } else {
                let size = egui::Vec2::new(self.config.window.width, self.config.window.height);
                if self.is_valid_window_size(size) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                } else {
                    tracing::warn!("Invalid initial window size, using platform fallback");
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.get_platform_safe_window_size()));
                }
            }
            
            if self.config.window.remember_position && !cfg!(target_os = "macos") {
                if let (Some(x), Some(y)) = (self.config.window.position_x, self.config.window.position_y) {
                    let pos = egui::Pos2::new(x, y);
                    if self.is_valid_window_position(pos) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                    } else {
                        tracing::warn!("Invalid initial window position, using system default");
                    }
                }
            } else if cfg!(target_os = "macos") {
                tracing::debug!("Skipping position setting on macOS for better system integration");
            }
            
            self.last_window_state = Some(window_state);
        });
    }

    fn detect_window_changes(&mut self, ctx: &egui::Context) {
        log_performance("detect_window_changes", || {
            let viewport_info = ctx.input(|i| i.viewport().clone());
            let current_size = viewport_info.inner_rect.map(|r| r.size()).unwrap_or_default();
            let current_maximized = viewport_info.maximized.unwrap_or(false);
            let current_pos = viewport_info.outer_rect.map(|r| r.min).unwrap_or_default();
            
            let mut config_changed = false;
            
            if let Some(ref last_state) = self.last_window_state {
                let size_threshold = 5.0;
                let pos_threshold = 10.0;
                
                if (current_size.x - last_state.width).abs() > size_threshold ||
                   (current_size.y - last_state.height).abs() > size_threshold {
                    if self.is_valid_window_size(current_size) {
                        self.config.window.width = current_size.x;
                        self.config.window.height = current_size.y;
                        config_changed = true;
                        tracing::debug!("Window size changed to {}x{}", current_size.x, current_size.y);
                    } else {
                        tracing::warn!("Detected invalid window size change, ignoring");
                    }
                }
                
                if current_maximized != last_state.maximized {
                    self.config.window.maximized = current_maximized;
                    config_changed = true;
                    tracing::debug!("Window maximized state changed to {}", current_maximized);
                }
                
                if self.config.window.remember_position {
                    if (current_pos.x - last_state.position_x.unwrap_or(0.0)).abs() > pos_threshold ||
                       (current_pos.y - last_state.position_y.unwrap_or(0.0)).abs() > pos_threshold {
                        if self.is_valid_window_position(current_pos) {
                            self.config.window.position_x = Some(current_pos.x);
                            self.config.window.position_y = Some(current_pos.y);
                            config_changed = true;
                            tracing::debug!("Window position changed to ({}, {})", current_pos.x, current_pos.y);
                        } else {
                            tracing::warn!("Detected invalid window position change, ignoring");
                        }
                    }
                }
            }
            
            if config_changed {
                self.last_window_state = Some(WindowState {
                    width: self.config.window.width,
                    height: self.config.window.height,
                    maximized: self.config.window.maximized,
                    position_x: self.config.window.position_x,
                    position_y: self.config.window.position_y,
                });
                
                if let Err(e) = self.config.save() {
                    tracing::error!("Failed to save window state changes: {}", e);
                    self.handle_window_operation_error("save_config", &e.to_string());
                } else {
                    tracing::debug!("Window state changes saved successfully");
                }
            }
        });
    }

    fn handle_viewport_events(&mut self, ctx: &egui::Context) {
        let viewport_info = ctx.input(|i| i.viewport().clone());
        let mut config_changed = false;
        
        if let Some(inner_size) = viewport_info.inner_rect {
            let new_width = inner_size.width();
            let new_height = inner_size.height();
            
            if (self.config.window.width - new_width).abs() > 1.0 || 
               (self.config.window.height - new_height).abs() > 1.0 {
                self.config.window.width = new_width;
                self.config.window.height = new_height;
                self.temp_config.window.width = new_width;
                self.temp_config.window.height = new_height;
                config_changed = true;
            }
        }
        
        if let Some(outer_pos) = viewport_info.outer_rect {
            if self.config.window.remember_position {
                let new_x = outer_pos.min.x;
                let new_y = outer_pos.min.y;
                
                if (self.config.window.position_x.unwrap_or(0.0) - new_x).abs() > 1.0 ||
                   (self.config.window.position_y.unwrap_or(0.0) - new_y).abs() > 1.0 {
                    self.config.window.position_x = Some(new_x);
                    self.config.window.position_y = Some(new_y);
                    self.temp_config.window.position_x = Some(new_x);
                    self.temp_config.window.position_y = Some(new_y);
                    config_changed = true;
                }
            }
        }
        
        if let Some(maximized) = viewport_info.maximized {
            if self.config.window.maximized != maximized {
                self.config.window.maximized = maximized;
                self.temp_config.window.maximized = maximized;
                config_changed = true;
            }
        }
        
        if config_changed {
            self.last_window_state = Some(WindowState::from_config(&self.config));
            
            if let Err(e) = self.config.save() {
                tracing::warn!("Failed to save window state changes: {}", e);
            } else {
                tracing::debug!("Window state changes saved");
            }
        }
    }

    fn is_system_dark_mode(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("reg")
                .args(&["query", "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize", "/v", "AppsUseLightTheme"])
                .output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return output_str.contains("0x0");
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("defaults")
                .args(&["read", "-g", "AppleInterfaceStyle"])
                .output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return output_str.trim() == "Dark";
            }
        }
        
        false
    }

    fn parse_hex_color(&self, hex: &str) -> std::result::Result<egui::Color32, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Invalid hex color length".to_string());
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component".to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component".to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component".to_string())?;
        
        Ok(egui::Color32::from_rgb(r, g, b))
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        if !self.config.ui.menu_preferences.show_menu_bar {
            return;
        }
        
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add(egui::Button::new("Open Repository").shortcut_text("Ctrl+O")).clicked() {
                        self.open_repository();
                        ui.close_menu();
                    }
                    
                    let has_repo = self.current_repository.is_some();
                    if ui.add_enabled(has_repo, egui::Button::new("Close Repository")).clicked() {
                        self.close_repository();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if !self.config.recent_repositories.repositories.is_empty() {
                        ui.menu_button("Recent Repositories", |ui| {
                            let recent_repos = self.config.recent_repositories.repositories.clone();
                            for (i, recent_repo) in recent_repos.iter().enumerate() {
                                if i >= self.config.recent_repositories.max_count { break; }
                                
                                let display_text = if recent_repo.name.len() > 30 {
                                    format!("{}...", &recent_repo.name[..27])
                                } else {
                                    recent_repo.name.clone()
                                };
                                
                                let button_text = format!("{}", display_text);
                                let tooltip_text = format!("{}\nPath: {}\nLast opened: {}", 
                                    recent_repo.name, 
                                    recent_repo.path.display(),
                                    recent_repo.last_opened.format("%Y-%m-%d %H:%M")
                                );
                                
                                if ui.button(button_text)
                                    .on_hover_text(tooltip_text)
                                    .clicked() 
                                {
                                    let path = recent_repo.path.clone();
                                    self.open_recent_repository(path);
                                    ui.close_menu();
                                }
                            }
                            
                            ui.separator();
                            if ui.button("Clear Recent").clicked() {
                                self.config.recent_repositories.clear();
                                if let Err(e) = self.config.save() {
                                    tracing::warn!("Failed to save config: {}", e);
                                }
                                ui.close_menu();
                            }
                        });
                        
                        ui.separator();
                    }
                    
                    if ui.add(egui::Button::new("Settings").shortcut_text("Ctrl+S")).clicked() {
                        self.temp_config = self.config.clone();
                        self.show_settings = true;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.add(egui::Button::new("Exit").shortcut_text("Ctrl+Q")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    let has_selection = false;
                    
                    ui.add_enabled(has_selection, egui::Button::new("Copy").shortcut_text("Ctrl+C"));
                    ui.add_enabled(has_selection, egui::Button::new("Cut").shortcut_text("Ctrl+X"));
                    ui.add_enabled(true, egui::Button::new("Paste").shortcut_text("Ctrl+V"));
                    
                    ui.separator();
                    
                    if ui.add(egui::Button::new("Select All").shortcut_text("Ctrl+A")).clicked() {
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    let theme_text = match self.config.theme.theme_type {
                        ThemeType::Light => "Switch to Dark Theme",
                        ThemeType::Dark => "Switch to Light Theme",
                        ThemeType::System => "Toggle Theme",
                    };
                    
                    if ui.button(theme_text).clicked() {
                        self.toggle_theme();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    let log_viewer_text = if self.show_log_viewer { "Hide Log Viewer" } else { "Show Log Viewer" };
                    if ui.button(log_viewer_text).clicked() {
                        self.show_log_viewer = !self.show_log_viewer;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    let menu_bar_text = if self.config.ui.menu_preferences.show_menu_bar { "Hide Menu Bar" } else { "Show Menu Bar" };
                    if ui.button(menu_bar_text).clicked() {
                        self.config.ui.menu_preferences.show_menu_bar = !self.config.ui.menu_preferences.show_menu_bar;
                        if let Err(e) = self.config.save() {
                            self.handle_error(e);
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Reset Layout").clicked() {
                        self.config.window.width = 1200.0;
                        self.config.window.height = 800.0;
                        self.config.window.maximized = false;
                        self.config.window.position_x = None;
                        self.config.window.position_y = None;
                        
                        if let Err(e) = self.config.save() {
                            self.handle_error(e);
                        } else {
                            self.add_notification(
                                "Layout reset to default".to_string(),
                                NotificationType::Success,
                                Some(2),
                            );
                        }
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if self.config.ui.menu_preferences.show_keyboard_shortcuts {
                        if ui.add(egui::Button::new("Keyboard Shortcuts").shortcut_text("F1")).clicked() {
                            self.show_shortcuts = true;
                            ui.close_menu();
                        }
                        
                        ui.separator();
                    }
                    
                    if ui.button("About Twiggy").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F10))) {
            self.config.ui.menu_preferences.show_menu_bar = !self.config.ui.menu_preferences.show_menu_bar;
            if let Err(e) = self.config.save() {
                self.handle_error(e);
            } else {
                let status = if self.config.ui.menu_preferences.show_menu_bar { "shown" } else { "hidden" };
                self.add_notification(
                    format!("Menu bar {}", status),
                    NotificationType::Info,
                    Some(2),
                );
            }
        }
        
        if !self.config.ui.menu_preferences.show_keyboard_shortcuts {
            return;
        }
        
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::O))) {
            self.open_repository();
        }
        
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S))) {
            self.temp_config = self.config.clone();
            self.show_settings = true;
        }
        
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Q))) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::O))) {
            if let Some(recent_repo) = self.config.recent_repositories.repositories.first() {
                let path = recent_repo.path.clone();
                self.open_recent_repository(path);
            }
        }
        
        for i in 1..=9 {
            let key = match i {
                1 => egui::Key::Num1,
                2 => egui::Key::Num2,
                3 => egui::Key::Num3,
                4 => egui::Key::Num4,
                5 => egui::Key::Num5,
                6 => egui::Key::Num6,
                7 => egui::Key::Num7,
                8 => egui::Key::Num8,
                9 => egui::Key::Num9,
                _ => continue,
            };
            
            if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, key))) {
                if let Some(recent_repo) = self.config.recent_repositories.repositories.get(i - 1) {
                    let path = recent_repo.path.clone();
                    self.open_recent_repository(path);
                }
            }
        }
        
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F1))) {
            self.show_shortcuts = true;
        }
    }

    fn render_help_dialogs(&mut self, ctx: &egui::Context) {
        if self.show_about {
            egui::Window::new("About Twiggy")
                .collapsible(false)
                .resizable(false)
                .default_size([400.0, 300.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading("🌿 Twiggy");
                        ui.add_space(5.0);
                        ui.label("Lightning-fast Git Visualization Tool");
                        ui.add_space(20.0);
                        
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Version:");
                            ui.label(env!("CARGO_PKG_VERSION"));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Author:");
                            ui.label("Twiggy Team");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Built with:");
                            ui.label("Rust + egui");
                        });
                        
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.label("Built with ❤️ using egui and Rust");
                        ui.label("© 2024 Twiggy Team");
                        
                        ui.add_space(20.0);
                        
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }

        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .collapsible(false)
                .resizable(true)
                .default_size([500.0, 400.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Application Shortcuts");
                        ui.add_space(10.0);
                        
                        egui::Grid::new("shortcuts_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Action");
                                ui.label("Shortcut");
                                ui.end_row();
                                
                                ui.separator();
                                ui.separator();
                                ui.end_row();
                                
                                ui.label("Open Repository");
                                ui.label("Ctrl+O");
                                ui.end_row();
                                
                                ui.label("Settings");
                                ui.label("Ctrl+,");
                                ui.end_row();
                                
                                ui.label("Exit Application");
                                ui.label("Ctrl+Q");
                                ui.end_row();
                                
                                ui.label("Show Shortcuts");
                                ui.label("F1");
                                ui.end_row();
                                
                                ui.label("Toggle Menu Bar");
                                ui.label("F10");
                                ui.end_row();
                                
                                ui.label("Close Dialog");
                                ui.label("Escape");
                                ui.end_row();
                                
                                ui.separator();
                                ui.separator();
                                ui.end_row();
                                
                                ui.label("Copy");
                                ui.label("Ctrl+C");
                                ui.end_row();
                                
                                ui.label("Select All");
                                ui.label("Ctrl+A");
                                ui.end_row();
                            });
                        
                        ui.add_space(20.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                self.show_shortcuts = false;
                            }
                        });
                    });
                });
        }
    }

    fn open_repository(&mut self) {
        tracing::info!("Repository opening requested");
        
        let mut dialog = rfd::FileDialog::new()
            .set_title("Open Git Repository - Select folder containing .git");
        
        let default_dir = self.get_default_directory_for_dialog();
        if let Some(dir) = default_dir {
            dialog = dialog.set_directory(dir);
        }
        
        if let Some(path) = dialog.pick_folder() {
            self.open_repository_path(path);
        }
    }
    
    fn get_default_directory_for_dialog(&self) -> Option<PathBuf> {
        if let Some(last_repo) = self.config.recent_repositories.repositories.first() {
            if last_repo.path.exists() {
                return Some(last_repo.path.clone());
            }
            if let Some(parent) = last_repo.path.parent() {
                if parent.exists() {
                    return Some(parent.to_path_buf());
                }
            }
        }
        
        if let Ok(home_dir) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let home_path = PathBuf::from(home_dir);
            if home_path.exists() {
                return Some(home_path);
            }
        }
        
        None
    }
    
    fn open_repository_path(&mut self, path: std::path::PathBuf) {
        if !path.exists() {
            tracing::error!("Repository path does not exist: {}", path.display());
            self.show_error_message(format!("Path does not exist: {}", path.display()));
            return;
        }
        
        if !path.is_dir() {
            tracing::error!("Repository path is not a directory: {}", path.display());
            self.show_error_message(format!("Path is not a directory: {}", path.display()));
            return;
        }
        
        self.repository_loading = true;
        
        match GitRepository::open(&path) {
            Ok(repo) => {
                let repo_name = repo.repository_name();
                tracing::info!("Repository opened: {}", repo_name);
                
                self.config.recent_repositories.add_repository(
                    path.clone(),
                    repo_name.clone(),
                );
                
                if let Err(e) = self.config.save() {
                    tracing::warn!("Failed to save config: {}", e);
                }
                
                self.current_repository = Some(repo);
                self.repository_loading = false;
                
                self.add_notification(
                    format!("Repository '{}' opened successfully", repo_name),
                    NotificationType::Success,
                    Some(3)
                );
            }
            Err(e) => {
                tracing::error!("Failed to open repository: {}", e);
                self.repository_loading = false;
                self.handle_error(e);
            }
        }
    }
    
    fn open_recent_repository(&mut self, path: std::path::PathBuf) {
        if path.exists() {
            self.open_repository_path(path);
        } else {
            tracing::warn!("Recent repository no longer exists: {}", path.display());
            self.config.recent_repositories.remove_repository(&path);
            
            if let Err(e) = self.config.save() {
                tracing::warn!("Failed to save config: {}", e);
            }
            
            self.add_notification(
                format!("Repository not found: {}", path.display()),
                NotificationType::Warning,
                Some(4)
            );
        }
    }
    
    fn close_repository(&mut self) {
        if let Some(ref repo) = self.current_repository {
            tracing::info!("Closing repository: {}", repo.repository_name());
            self.current_repository = None;
            
            self.add_notification(
                "Repository closed".to_string(),
                NotificationType::Info,
                Some(2)
            );
        }
    }
    
    fn render_repository_info(&self, ui: &mut egui::Ui) {
        if let Some(ref repo) = self.current_repository {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.strong(format!("Repository: {}", repo.repository_name()));
                    ui.label(format!("Path: {}", repo.path().display()));
                    
                    if let Some(branch) = repo.current_branch() {
                        ui.label(format!("Branch: {}", branch));
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "Branch: Detached HEAD");
                    }
                    
                    match repo.commit_count() {
                        Ok(count) => ui.label(format!("Commits: {}", count)),
                        Err(_) => ui.colored_label(egui::Color32::GRAY, "Commits: Unable to count"),
                    };
                });
            });
        } else if self.repository_loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading repository...");
            });
        } else {
            ui.label("No repository open");
        }
    }

    fn show_error_message(&mut self, message: String) {
        self.add_notification(
            message,
            NotificationType::Error,
            Some(5),
        );
    }
    
    fn toggle_theme(&mut self) {
        self.temp_config.theme.theme_type = match self.temp_config.theme.theme_type {
            ThemeType::Light => ThemeType::Dark,
            ThemeType::Dark => ThemeType::Light,
            ThemeType::System => ThemeType::Light,
        };
        
        self.config.theme.theme_type = self.temp_config.theme.theme_type.clone();
        
        if let Err(e) = self.config.save() {
            self.handle_error(e);
        } else {
            self.add_notification(
                format!("Theme changed to {:?}", self.config.theme.theme_type),
                NotificationType::Success,
                Some(2),
            );
        }
    }
}

impl eframe::App for TwiggyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        log_performance("frame_update", || {
            self.update_performance_metrics();
            self.auto_save_config_if_needed();
            self.cleanup_old_notifications();
            
            self.handle_viewport_events(ctx);
            self.detect_window_changes(ctx);
            
            self.apply_theme_to_context(ctx);
            
            self.handle_keyboard_shortcuts(ctx);
            self.render_menu_bar(ctx);
            
            self.render_error_dialog(ctx);
            self.render_notifications(ctx);
            self.render_settings_dialog(ctx, frame);
            self.render_help_dialogs(ctx);
            
            if self.show_log_viewer {
                egui::Window::new("Log Viewer")
                    .resizable(true)
                    .default_size([800.0, 600.0])
                    .show(ctx, |ui| {
                        if let Err(e) = self.log_viewer.render(ui) {
                            self.add_notification(
                                format!("Log viewer error: {}", e),
                                NotificationType::Error,
                                Some(5),
                            );
                        }
                    });
                
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_log_viewer = false;
                }
            }
            
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("🌿 Twiggy");
                    ui.label("Lightning-fast Git Visualization Tool");
                    ui.add_space(20.0);
                    
                    if let Some(ref repo) = self.current_repository {
                        ui.label("Phase 10: Repository Management - ✅ Active");
                        ui.add_space(10.0);
                        self.render_repository_info(ui);
                    } else {
                        ui.label("Phase 10: Repository Management - Ready");
                        ui.add_space(10.0);
                        ui.label("Open a repository from the File menu to get started");
                    }
                    
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
                        if ui.button("Test Error").clicked() {
                            let test_error = TwiggyError::Application {
                                message: "This is a test error for demonstration".to_string(),
                            };
                            self.handle_error(test_error);
                        }
                        
                        if ui.button("Test Notification").clicked() {
                            self.add_notification(
                                "This is a test notification".to_string(),
                                NotificationType::Info,
                                Some(5),
                            );
                        }
                        
                        if ui.button("Settings").clicked() {
                            self.temp_config = self.config.clone();
                            self.show_settings = true;
                        }
                        
                        if ui.button("Reset Config").clicked() {
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
        });
        
        if self.performance_metrics.frame_count % 300 == 0 {
            log_memory_usage("periodic_check");
        }
    }
}