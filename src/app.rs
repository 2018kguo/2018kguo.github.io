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
    show_notes_mobile: bool,
    #[serde(skip)]
    life: Life,
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
        // Bundle JetBrains Mono Medium as the monospace face — heavier stroke than egui's default Hack
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "jetbrains_mono".to_owned(),
            egui::FontData::from_static(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/JetBrainsMono-Medium.ttf"
            ))),
        );
        fonts
            .families
            .entry(Monospace)
            .or_default()
            .insert(0, "jetbrains_mono".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Set up fonts and styles
        let mut visuals = egui::Visuals::light();
        // Darker body text — egui's default light gray reads as thin/faint on white
        visuals.override_text_color = Some(egui::Color32::from_gray(45));
        cc.egui_ctx.set_visuals(visuals);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(19.0, Monospace)),
            (TextStyle::Body, FontId::new(17.0, Monospace)),
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
            life: Life::default(),
        }
    }

    fn load_blog_posts() -> Vec<BlogPost> {
        include_blog_posts!(
            "ramp_career",
            "Notes after Ramp",
            "2026",
            "llm_math",
            "A layman's view of LLMs",
            "2026",
            "emulators",
            "Feature emulators",
            "2026",
            "reading_list",
            "Reading List",
            "",
            "pub_links",
            "Misc Links",
            "",
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
    ui.label("I'm a software engineer based in NYC. I was previously at Ramp and am now working on something new.");
    ui.label("I strive to build performant and composable software. I've recently been interested in event-driven systems, Rust, and programming languages.");
}

fn render_links(ui: &mut egui::Ui) {
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
    for (index, post) in app.blog_posts.iter().enumerate() {
        if ui.button(&post.title).clicked() {
            app.current_post_index = Some(index);
        }
        if &post.date.len() > &0 {
            ui.label(RichText::new(&post.date).small().weak());
        }
        ui.add_space(4.0);
    }
}

fn show_blog_content(app: &mut TemplateApp, ui: &mut egui::Ui) {
    if let Some(index) = app.current_post_index {
        let post = &app.blog_posts[index];
        ui.horizontal(|ui| {
            ui.add_space(10.0); // Adds 10 pixels of horizontal space
            ui.vertical(|ui| {
                ui.heading(&post.title);
                if &post.date.len() > &0 {
                    ui.label(RichText::new(&post.date).italics());
                }
            });
        });
        ui.add_space(2.0);

        // ponytail: cap line length (~760px) for readability; full-width monospace is too wide to read
        ui.set_max_width(760.0);
        ui.spacing_mut().item_spacing.y = 8.0; // a little more air between paragraphs
        let mut cache = CommonMarkCache::default();
        CommonMarkViewer::new("viewer").show(ui, &mut cache, &post.content);
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a post to view");
        });
    }
}

// A faint Game of Life drifting behind the windows, on a dot grid.
#[derive(Default)]
struct Life {
    cols: usize,
    rows: usize,
    cells: Vec<bool>,
    last_step: f64,
    rng: u64,
    prev_pop: usize,
    stale: u32,
}

impl Life {
    fn next_rng(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    fn seed_random(&mut self) {
        let len = self.cells.len();
        for i in 0..len {
            self.cells[i] = (self.next_rng() & 0xFF) < 46; // ~18% density
        }
        self.prev_pop = self.cells.iter().filter(|&&b| b).count();
        self.stale = 0;
    }

    fn ensure(&mut self, cols: usize, rows: usize, seed: u64) {
        if self.cols != cols || self.rows != rows {
            self.cols = cols;
            self.rows = rows;
            self.cells = vec![false; cols * rows];
            if self.rng == 0 {
                self.rng = seed | 1;
            }
            self.seed_random();
        }
    }

    fn step(&mut self) {
        let (cols, rows) = (self.cols, self.rows);
        if cols == 0 || rows == 0 {
            return;
        }
        let mut next = vec![false; cols * rows];
        for y in 0..rows {
            for x in 0..cols {
                let mut n = 0;
                for dy in 0..3 {
                    for dx in 0..3 {
                        if dx == 1 && dy == 1 {
                            continue;
                        }
                        let nx = (x + cols + dx - 1) % cols;
                        let ny = (y + rows + dy - 1) % rows;
                        if self.cells[ny * cols + nx] {
                            n += 1;
                        }
                    }
                }
                let alive = self.cells[y * cols + x];
                next[y * cols + x] = matches!((alive, n), (true, 2) | (true, 3) | (false, 3));
            }
        }
        self.cells = next;

        // Re-seed when the board dies out or stagnates (still lifes / oscillators).
        let pop = self.cells.iter().filter(|&&b| b).count();
        if pop == self.prev_pop {
            self.stale += 1;
        } else {
            self.stale = 0;
        }
        self.prev_pop = pop;
        if pop == 0 || self.stale > 18 {
            self.seed_random();
        }
    }
}

const LIFE_CELL: f32 = 26.0;
const LIFE_STEP_SECS: f64 = 0.6;

fn paint_background(life: &mut Life, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
    let cols = (rect.width() / LIFE_CELL).ceil() as usize + 1;
    let rows = (rect.height() / LIFE_CELL).ceil() as usize + 1;
    let now = ctx.input(|i| i.time);
    let seed = (now * 1_000_000.0) as u64 ^ 0x9E37_79B9_7F4A_7C15;
    life.ensure(cols, rows, seed);

    if life.last_step == 0.0 {
        life.last_step = now;
    }
    if now - life.last_step >= LIFE_STEP_SECS {
        life.step();
        life.last_step = now;
    }

    // Base wallpaper.
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(250));

    // Live cells: barely-there rounded squares.
    let cell_col = egui::Color32::from_gray(234);
    for y in 0..rows {
        for x in 0..cols {
            if life.cells[y * cols + x] {
                let p = rect.min + egui::vec2(x as f32 * LIFE_CELL, y as f32 * LIFE_CELL);
                painter.rect_filled(
                    egui::Rect::from_min_size(p, egui::vec2(LIFE_CELL, LIFE_CELL)).shrink(3.5),
                    2.0,
                    cell_col,
                );
            }
        }
    }

    // Dot grid on top.
    let dot = egui::Color32::from_gray(219);
    for y in 0..rows {
        for x in 0..cols {
            let p = rect.min + egui::vec2(x as f32 * LIFE_CELL, y as f32 * LIFE_CELL);
            painter.circle_filled(p, 1.0, dot);
        }
    }

    ctx.request_repaint_after(std::time::Duration::from_secs_f64(LIFE_STEP_SECS));
}

// Desktop layout
fn desktop_layout(app: &mut TemplateApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        paint_background(&mut app.life, ctx, ui.painter(), ui.max_rect());
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
                                            show_blog_content(app, ui);
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
                show_blog_content(app, ui);
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
