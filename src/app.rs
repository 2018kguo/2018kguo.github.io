use egui::{FontFamily::*, FontId, RichText, TextStyle};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BlogPost {
    title: String,
    content: String,
    date: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    blog_posts: Vec<BlogPost>,
    current_post_index: Option<usize>,
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

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            blog_posts: Vec::new(),
            current_post_index: None,
        }
    }
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
            (TextStyle::Small, FontId::new(10.0, Monospace)),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        // Load blog posts
        let blog_posts = Self::load_blog_posts();
        let len_blog_posts = blog_posts.len();

        Self {
            blog_posts: blog_posts,
            current_post_index: if len_blog_posts > 0 { Some(0) } else { None }
        }
    }

    fn load_blog_posts() -> Vec<BlogPost> {
        include_blog_posts!(
            "postgres", "Postgres", "2024-09-16"
        )
    }
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
                let blog_window = make_window_vertical("Blog Posts", false);

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

                links_window.show(ctx, |ui| {
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

                blog_window.show(ctx, |ui| {
                    make_frame_with_padding(20.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;
                        ui.strong("Notes");
                        ui.separator();
                        ui.add_space(8.0); // Add small vertical space after separator
        
                        if self.blog_posts.is_empty() {
                            ui.label("Nothing here yet!");
                        } else {
                            ui.horizontal(|ui| {
                                egui::SidePanel::left("blog_list_panel")
                                    .resizable(false)
                                    .default_width(150.0)
                                    .show_inside(ui, |ui| {
                                        ui.vertical(|ui| {
                                            for (index, post) in self.blog_posts.iter().enumerate() {
                                                if ui.button(&post.title).clicked() {
                                                    self.current_post_index = Some(index);
                                                }
                                                ui.label(RichText::new(&post.date).small().weak());
                                                ui.add_space(4.0);
                                            }
                                        });
                                    });
        
                                ui.add_space(16.0); // Add small horizontal space between sidebar and content
        
                                ui.vertical(|ui| {
                                    if let Some(index) = self.current_post_index {
                                        let post = &self.blog_posts[index];
                                        ui.heading(&post.title);
                                        ui.label(RichText::new(&post.date).italics());
                                        ui.add_space(5.0);
                                        egui::ScrollArea::vertical().min_scrolled_height(1200.0).show(ui, |ui| {
                                            render_markdown(ui, &post.content);
                                        });
                                    } else {
                                        ui.centered_and_justified(|ui| {
                                            ui.label("Select a post to view");
                                        });
                                    }
                                });
                            });
                        }
                    });
                });
            }); 
        });
    }
}

fn render_markdown(ui: &mut egui::Ui, markdown: &str) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);

    let mut current_heading_level = 6;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                //current_heading_level = level;
            }
            Event::End(TagEnd::Heading { .. }) => {
                current_heading_level = 6;
                ui.add_space(8.0);
            }
            Event::Start(Tag::Paragraph) => {
                ui.add_space(4.0);
            }
            Event::End(TagEnd::Paragraph) => {
                ui.add_space(8.0);
            }
            Event::Text(text) => {
                let text_style = match current_heading_level {
                    1 => TextStyle::Heading,
                    2 => TextStyle::Name("Heading2".into()),
                    3 => TextStyle::Name("Heading3".into()),
                    4 => TextStyle::Name("Heading4".into()),
                    5 => TextStyle::Name("Heading5".into()),
                    _ => TextStyle::Body,
                };
                ui.label(RichText::new(text.to_string()).text_style(text_style));
            }
            Event::Start(Tag::List { .. }) => {
                ui.add_space(4.0);
            }
            Event::End(TagEnd::List { .. }) => {
                ui.add_space(8.0);
            }
            Event::Start(Tag::Item) => {
                ui.horizontal(|ui| {
                    ui.label("• ");
                });
            }
            Event::SoftBreak => {
                ui.add_space(4.0);
            }
            Event::HardBreak => {
                ui.add_space(8.0);
            }
            _ => {}
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
        .resizable([true, false])
        .title_bar(show_title)
        .default_size([400.0, 500.0])  // Taller default size
        .min_width(200.0)  // Minimum width to prevent too narrow windows
        .min_height(300.0)  // Minimum height to ensure usability
}

fn make_frame_with_padding(padding: f32) -> egui::Frame {
    egui::Frame::none().inner_margin(egui::Margin::same(padding))
}