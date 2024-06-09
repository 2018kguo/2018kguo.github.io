use egui::TextStyle;
use egui::{FontFamily::*, FontId};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(18.0, Monospace)),
            (TextStyle::Body, FontId::new(16.0, Monospace)),
            (TextStyle::Monospace, FontId::new(14.0, Monospace)),
            (TextStyle::Button, FontId::new(14.0, Monospace)),
            (TextStyle::Small, FontId::new(10.0, Monospace)),
        ]
        .into();
        cc.egui_ctx.set_style(style);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::none().inner_margin(egui::Margin {
                left: 100.0,
                right: 100.0,
                top: 0.0,
                bottom: 0.0,
            }).show(ui, |_ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                let intro_window = make_window("Intro", false);
                let links_window = make_window("Links", false);
                intro_window.show(ctx, |ui| {
                    make_frame_with_padding(20.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;
                        ui.strong("Kevin Guo");
                        ui.separator();
                        ui.label("I'm a software engineer based in NYC. I currently work at Ramp, where I build tools to help finance teams manage their expenses and automate the tedious parts of their job.");
                        ui.label("");
                        ui.label("I strive to build performant and composable software. I've recently been interested in event-driven systems, Rust, and programming languages.");
                    });
                });
                links_window.show(ctx, |
                ui| {
                    make_frame_with_padding(20.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;
                        ui.strong("Links");
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.weak("Email: ");
                            ui.hyperlink_to("kev.guo123@gmail.com", "mailto:kev.guo123@gmail.com");
                        });
                        ui.horizontal(|ui| {
                            ui.weak("LinkedIn: ");
                            ui.hyperlink_to("@2018kguo", "https://www.linkedin.com/in/2018kguo/");
                        });
                        ui.horizontal(|ui| {
                            ui.weak("GitHub: ");
                            ui.hyperlink_to("@2018kguo", "https://www.github.com/2018kguo/");
                        });
                        ui.horizontal(|ui| {
                            ui.weak("Medium: ");
                            ui.hyperlink_to("@2018kguo", "https://2018kguo.medium.com/");
                        });
                        ui.horizontal(|ui| {
                            ui.weak("Resume: ");
                            ui.hyperlink_to("link", "https://drive.google.com/file/d/1aL_SNToa_CmD92qWKYF2ouFmAGUMPb2Y/view");
                        });
                    });
                });
            });
        });
    }
}

fn make_window(title: &str, show_title: bool) -> egui::Window<'_> {
    egui::Window::new(title.to_string())
        .collapsible(false)
        .resizable([true, false])
        .title_bar(show_title)
        .constrain(true)
        .default_size([600.0, 300.0])
}

fn make_frame_with_padding(padding: f32) -> egui::Frame {
    egui::Frame::none().inner_margin(egui::Margin::same(padding))
}
