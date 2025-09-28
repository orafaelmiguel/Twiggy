use eframe::egui;
use crate::error::TwiggyError;
use std::time::Instant;

#[derive(Debug)]
pub struct ErrorDialog {
    pub error: TwiggyError,
    pub timestamp: Instant,
    pub show_details: bool,
    pub is_visible: bool,
}

impl ErrorDialog {
    pub fn new(error: TwiggyError) -> Self {
        Self {
            error,
            timestamp: Instant::now(),
            show_details: false,
            is_visible: true,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) -> ErrorDialogResponse {
        let mut response = ErrorDialogResponse::None;

        if !self.is_visible {
            return response;
        }

        egui::Window::new("⚠️ Application Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(450.0);
                ui.set_max_width(600.0);

                ui.vertical(|ui| {
                    ui.add_space(10.0);

                    let error_icon = match self.error.error_code() {
                        1000..=1999 => "🔧",
                        2000..=2999 => "📁", 
                        3000..=3999 => "⚙️",
                        4000..=4999 => "🖥️",
                        _ => "❌",
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(error_icon).size(24.0));
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(self.error.user_message())
                                .size(14.0)
                                .color(egui::Color32::from_rgb(220, 50, 50)));
                            
                            ui.label(egui::RichText::new(format!("Error Code: {}", self.error.error_code()))
                                .size(11.0)
                                .color(egui::Color32::GRAY));
                        });
                    });

                    ui.add_space(15.0);

                    if let Some(suggested_action) = self.error.suggested_action() {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("💡 Suggested Solution:")
                                    .strong()
                                    .color(egui::Color32::from_rgb(70, 130, 180)));
                                ui.label(suggested_action);
                            });
                        });
                        ui.add_space(10.0);
                    }

                    ui.collapsing("🔍 Technical Details", |ui| {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Error Type:")
                                    .strong());
                                ui.label(format!("{:?}", self.error));
                                
                                ui.add_space(5.0);
                                
                                ui.label(egui::RichText::new("Timestamp:")
                                    .strong());
                                ui.label(format!("{:?}", self.timestamp));
                                
                                ui.add_space(5.0);
                                
                                ui.label(egui::RichText::new("Recoverable:")
                                    .strong());
                                ui.label(if self.error.is_recoverable() { "Yes" } else { "No" });
                            });
                        });
                    });

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if self.error.is_recoverable() {
                            if ui.button("🔄 Try to Recover").clicked() {
                                response = ErrorDialogResponse::TryRecover;
                            }
                        }

                        if ui.button("📋 Copy Details").clicked() {
                            let details = format!(
                                "Twiggy Error Report\n\
                                Error Code: {}\n\
                                Message: {}\n\
                                Timestamp: {:?}\n\
                                Recoverable: {}\n\
                                Technical Details: {:?}",
                                self.error.error_code(),
                                self.error.user_message(),
                                self.timestamp,
                                self.error.is_recoverable(),
                                self.error
                            );
                            ui.output_mut(|o| o.copied_text = details);
                            response = ErrorDialogResponse::CopiedToClipboard;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✅ OK").clicked() {
                                response = ErrorDialogResponse::Dismissed;
                                self.is_visible = false;
                            }
                        });
                    });

                    ui.add_space(5.0);
                });
            });

        response
    }

    pub fn dismiss(&mut self) {
        self.is_visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorDialogResponse {
    None,
    TryRecover,
    CopiedToClipboard,
    Dismissed,
}

pub struct ErrorNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: Instant,
    pub auto_dismiss_seconds: Option<u32>,
    pub is_visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

impl ErrorNotification {
    pub fn new(message: String, notification_type: NotificationType, auto_dismiss_seconds: Option<u32>) -> Self {
        Self {
            message,
            notification_type,
            timestamp: Instant::now(),
            auto_dismiss_seconds,
            is_visible: true,
        }
    }

    pub fn should_auto_dismiss(&self) -> bool {
        if let Some(dismiss_time) = self.auto_dismiss_seconds {
            self.timestamp.elapsed().as_secs() >= dismiss_time as u64
        } else {
            false
        }
    }

    pub fn dismiss(&mut self) {
        self.is_visible = false;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.is_visible {
            return false;
        }

        let (bg_color, text_color, icon) = match self.notification_type {
            NotificationType::Info => (egui::Color32::from_rgb(70, 130, 180), egui::Color32::WHITE, "ℹ️"),
            NotificationType::Warning => (egui::Color32::from_rgb(255, 165, 0), egui::Color32::BLACK, "⚠️"),
            NotificationType::Error => (egui::Color32::from_rgb(220, 50, 50), egui::Color32::WHITE, "❌"),
            NotificationType::Success => (egui::Color32::from_rgb(50, 180, 50), egui::Color32::WHITE, "✅"),
        };

        let frame = egui::Frame::default()
            .fill(bg_color)
            .rounding(8.0)
            .inner_margin(egui::style::Margin::same(12.0))
            .shadow(egui::epaint::Shadow::small_dark());

        let mut dismissed = false;

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(16.0));
                
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&self.message)
                        .color(text_color)
                        .size(12.0));
                    
                    if let Some(dismiss_time) = self.auto_dismiss_seconds {
                        let elapsed = self.timestamp.elapsed().as_secs();
                        let remaining = dismiss_time as u64 - elapsed.min(dismiss_time as u64);
                        if remaining > 0 {
                            ui.label(egui::RichText::new(format!("Auto-dismiss in {}s", remaining))
                                .color(text_color)
                                .size(10.0));
                        }
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        dismissed = true;
                    }
                });
            });
        });

        if dismissed {
            self.dismiss();
        }

        true
    }
}

pub struct NotificationManager {
    notifications: Vec<ErrorNotification>,
    max_notifications: usize,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            max_notifications: 5,
        }
    }
}

impl NotificationManager {
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType, auto_dismiss_seconds: Option<u32>) {
        let notification = ErrorNotification::new(message, notification_type, auto_dismiss_seconds);
        self.notifications.push(notification);

        if self.notifications.len() > self.max_notifications {
            self.notifications.remove(0);
        }
    }

    pub fn update(&mut self) {
        self.notifications.retain(|notification| {
            !notification.should_auto_dismiss() && notification.is_visible
        });
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        if self.notifications.is_empty() {
            return;
        }

        egui::Area::new("error_notifications")
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-15.0, 15.0))
            .show(ctx, |ui| {
                ui.set_max_width(350.0);
                
                for notification in &mut self.notifications {
                    notification.render(ui);
                    ui.add_space(8.0);
                }
            });
    }

    pub fn clear_all(&mut self) {
        self.notifications.clear();
    }

    pub fn notification_count(&self) -> usize {
        self.notifications.len()
    }
}