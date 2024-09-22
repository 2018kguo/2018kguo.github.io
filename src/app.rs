use egui::{FontFamily::*, FontId, RichText, TextStyle};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BlogPost {
    title: String,
    content: String,
    date: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct TemplateApp {
    blog_posts: Vec<BlogPost>,
    current_post_index: Option<usize>,
    current_page: usize,
    current_content_page: usize,
    show_notes_mobile: bool,
}

// Macro to include blog posts
macro_rules! include_blog_posts {
    ($($filename:expr, $title:expr, $date:expr),+) => {
        vec![
            $(
                BlogPost {
                    title: $title.to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/blog_posts/", $filename, ".md")).to_string(),
                    date: $date.to_string(),
                },
            )+
        ]
    };
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up fonts and styles
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(18.0, Monospace)),
            (TextStyle::Body, FontId::new(16.0, Monospace)),
            (TextStyle::Monospace, FontId::new(14.0, Monospace)),
            (TextStyle::Button, FontId::new(14.0, Monospace)),
            (TextStyle::Small, FontId::new(13.0, Monospace)),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        // Load blog posts
        let blog_posts = Self::load_blog_posts();
        let len_blog_posts = blog_posts.len();

        Self {
            blog_posts,
            current_post_index: if len_blog_posts > 0 { Some(0) } else { None },
            show_notes_mobile: false,
            current_content_page: 0,
            current_page: 0,
        }
    }

    fn load_blog_posts() -> Vec<BlogPost> {
        include_blog_posts!(
            "reading_list",
            "Reading List",
            "2024",
            "postgres",
            "Postgres",
            "2024",
            "interpreters",
            "Interpreters",
            "2024"
        )
    }
}

fn is_mobile_or_small_screen(ctx: &egui::Context) -> bool {
    let available_rect = ctx.available_rect();
    let width = available_rect.width();
    let height = available_rect.height();

    // Define thresholds for what you consider "mobile" or "small screen"
    const MOBILE_WIDTH_THRESHOLD: f32 = 600.0;
    const MOBILE_HEIGHT_THRESHOLD: f32 = 800.0;

    // Check if either dimension is below the threshold
    width < MOBILE_WIDTH_THRESHOLD || height < MOBILE_HEIGHT_THRESHOLD
}

// Shared content functions
fn render_intro(ui: &mut egui::Ui) {
    ui.heading("About Me");
    ui.label("I'm a software engineer based in NYC. I currently work at Ramp, where I build tools to help finance teams manage their expenses and automate the tedious parts of their job.");
    ui.label("I strive to build performant and composable software. I've recently been interested in event-driven systems, Rust, and programming languages.");
}

fn render_links(ui: &mut egui::Ui) {
    ui.heading("Links");
    add_link(
        ui,
        "Email",
        "kev.guo123@gmail.com",
        "mailto:kev.guo123@gmail.com",
    );
    add_link(
        ui,
        "LinkedIn",
        "@2018kguo",
        "https://www.linkedin.com/in/2018kguo/",
    );
    add_link(
        ui,
        "GitHub",
        "@2018kguo",
        "https://www.github.com/2018kguo/",
    );
    add_link(ui, "Medium", "@2018kguo", "https://2018kguo.medium.com/");
    add_link(
        ui,
        "Resume",
        "link",
        "https://drive.google.com/file/d/1aL_SNToa_CmD92qWKYF2ouFmAGUMPb2Y/view",
    );
}

fn add_link(ui: &mut egui::Ui, label: &str, text: &str, url: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("{}: ", label));
        ui.hyperlink_to(text, url);
    });
}

fn show_blog_list(app: &mut TemplateApp, ui: &mut egui::Ui) {
    const ITEMS_PER_PAGE: usize = 5;

    let total_pages = (app.blog_posts.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
    let start_index = app.current_page * ITEMS_PER_PAGE;
    let end_index = (start_index + ITEMS_PER_PAGE).min(app.blog_posts.len());

    for (index, post) in app.blog_posts[start_index..end_index].iter().enumerate() {
        if ui.button(&post.title).clicked() {
            app.current_post_index = Some(start_index + index);
            app.current_content_page = 0;
        }
        ui.label(RichText::new(&post.date).small().weak());
        ui.add_space(4.0);
    }

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if total_pages > 1 {
            if ui.button("◀").clicked() && app.current_page > 0 {
                app.current_page -= 1;
            }
            ui.label(format!("{}/{}", app.current_page + 1, total_pages));
            if ui.button("▶").clicked() && app.current_page < total_pages - 1 {
                app.current_page += 1;
            }
        }
    });
}

fn show_blog_content(app: &mut TemplateApp, ui: &mut egui::Ui, lines_per_page: usize) {
    if let Some(index) = app.current_post_index {
        let post = &app.blog_posts[index];
        ui.heading(&post.title);
        ui.label(RichText::new(&post.date).italics());
        ui.add_space(5.0);

        let content_lines: Vec<&str> = post.content.split('\n').collect();
        let total_pages = (content_lines.len() + lines_per_page - 1) / lines_per_page;
        let start_line = app.current_content_page * lines_per_page;
        let end_line = (start_line + lines_per_page).min(content_lines.len());

        egui::ScrollArea::vertical()
            .min_scrolled_height(1000.0)
            .show(ui, |ui| {
                let content_slice = &content_lines[start_line..end_line];
                let page_content = content_slice.join("\n");
                let mut cache = CommonMarkCache::default();
                CommonMarkViewer::new("viewer").show(ui, &mut cache, &page_content);
            });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if total_pages > 1 {
                if ui.button("◀").clicked() && app.current_content_page > 0 {
                    app.current_content_page -= 1;
                }
                ui.label(format!(
                    "Page {}/{}",
                    app.current_content_page + 1,
                    total_pages
                ));
                if ui.button("▶").clicked() && app.current_content_page < total_pages - 1 {
                    app.current_content_page += 1;
                }
            }
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a post to view");
        });
    }
}

// Desktop layout
fn desktop_layout(app: &mut TemplateApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: 100.0,
                    right: 100.0,
                    top: 0.0,
                    bottom: 0.0,
                })
                .show(ui, |_ui| {
                    let intro_window = make_window("Intro", false);
                    let links_window = make_window("Links", false);
                    let blog_window = make_window_vertical("Notes", false);

                    intro_window.show(ctx, |ui| {
                        make_frame_with_padding(20.0).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.strong("Kevin Guo");
                            ui.separator();
                            render_intro(ui);
                        });
                    });

                    links_window.show(ctx, |ui| {
                        make_frame_with_padding(20.0).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.strong("Links");
                            ui.separator();
                            render_links(ui);
                        });
                    });

                    blog_window.show(ctx, |ui| {
                        make_frame_with_padding(20.0).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.strong("Notes");
                            ui.separator();
                            ui.add_space(8.0);

                            if app.blog_posts.is_empty() {
                                ui.label("Nothing here yet!");
                            } else {
                                egui::SidePanel::left("blog_list_panel")
                                    .resizable(false)
                                    .default_width(150.0)
                                    .min_width(150.0)
                                    .show_inside(ui, |ui| {
                                        egui::ScrollArea::vertical().show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                show_blog_list(app, ui);
                                            });
                                        });
                                    });

                                ui.add_space(16.0);

                                egui::CentralPanel::default().show_inside(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .min_scrolled_height(400.0)
                                        .show(ui, |ui| {
                                            show_blog_content(app, ui, 30);
                                        });
                                });
                            }
                        });
                    });
                });
        });
    });
}

// Mobile layout
fn mobile_layout(app: &mut TemplateApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("mobile_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Kevin Guo");
            ui.add_space(ui.available_width() - 50.0);
            let button_text = if app.show_notes_mobile {
                "Close"
            } else {
                "Notes"
            };
            if ui.button(button_text).clicked() {
                app.show_notes_mobile = !app.show_notes_mobile;
            }
        });
    });

    if app.show_notes_mobile {
        show_notes_panel(app, ctx);
    } else {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                render_intro(ui);
                ui.add_space(20.0);
                render_links(ui);
            });
        });
    }
}

fn show_notes_panel(app: &mut TemplateApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Notes");
            ui.add_space(10.0);

            if app.blog_posts.is_empty() {
                ui.label("Nothing here yet!");
            } else {
                show_blog_list(app, ui);
                ui.add_space(20.0);
                show_blog_content(app, ui, 15); // Using fewer lines per page for mobile
            }
        });
    });
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        if !is_mobile_or_small_screen(ctx) {
            desktop_layout(self, ctx);
        } else {
            mobile_layout(self, ctx);
        }
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

fn make_window_vertical(title: &str, show_title: bool) -> egui::Window<'_> {
    egui::Window::new(title.to_string())
        .collapsible(false)
        .resizable(true)
        .title_bar(show_title)
        .default_size([1000.0, 800.0]) // Taller default size
        .min_width(1000.0) // Minimum width to prevent too narrow windows
        .min_height(800.0) // Minimum height to ensure usability
}

fn make_frame_with_padding(padding: f32) -> egui::Frame {
    egui::Frame::none().inner_margin(egui::Margin::same(padding))
}
