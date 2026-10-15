use egui::{FontFamily::*, FontId, RichText, TextStyle};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BlogPost {
    title: String,
    content: String,
    date: String,
}

#[derive(Default)]
enum MobilePage {
    #[default]
    About,
    Notes,
    Post,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct TemplateApp {
    blog_posts: Vec<BlogPost>,
    current_post_index: Option<usize>,
    #[serde(skip)]
    mobile_page: MobilePage,
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
        configure_style(&cc.egui_ctx);

        let blog_posts = Self::load_blog_posts();
        let current_post_index = if blog_posts.is_empty() { None } else { Some(0) };

        Self {
            blog_posts,
            current_post_index,
            mobile_page: MobilePage::About,
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

fn configure_style(ctx: &egui::Context) {
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
    ctx.set_fonts(fonts);

    // Set up fonts and styles
    ctx.set_visuals(egui::Visuals::light());
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(19.0, Monospace)),
        (TextStyle::Body, FontId::new(17.0, Monospace)),
        (TextStyle::Monospace, FontId::new(14.0, Monospace)),
        (TextStyle::Button, FontId::new(14.0, Monospace)),
        (TextStyle::Small, FontId::new(13.0, Monospace)),
    ]
    .into();
    ctx.set_style(style);
}

fn is_mobile_or_small_screen(ctx: &egui::Context) -> bool {
    // The desktop notes window needs room for both the list and the article.
    ctx.screen_rect().width() < 1100.0
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
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}: ", label));
        ui.hyperlink_to(text, url);
    });
}

fn show_blog_list(app: &mut TemplateApp, ui: &mut egui::Ui) -> bool {
    let mut selected = false;
    for (index, post) in app.blog_posts.iter().enumerate() {
        if ui.add(egui::Button::new(&post.title).wrap()).clicked() {
            app.current_post_index = Some(index);
            selected = true;
        }
        if !post.date.is_empty() {
            ui.label(RichText::new(&post.date).small().weak());
        }
        ui.add_space(4.0);
    }
    selected
}

fn show_blog_content(app: &mut TemplateApp, ui: &mut egui::Ui) {
    if let Some(index) = app.current_post_index {
        let post = &app.blog_posts[index];
        // set_max_width can expand the UI, so never exceed the viewport's remaining width.
        ui.set_max_width(ui.available_width().min(760.0));
        ui.heading(&post.title);
        if !post.date.is_empty() {
            ui.label(RichText::new(&post.date).small().weak());
        }
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing.y = 8.0; // a little more air between paragraphs
        let mut cache = CommonMarkCache::default();
        // Keep the Markdown title for readers of the source without displaying it twice.
        let title = format!("# {}\n\n", post.title);
        let content = post.content.strip_prefix(&title).unwrap_or(&post.content);
        CommonMarkViewer::new(("viewer", index)).show(ui, &mut cache, content);
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

        // Re-seed when the board dies out or stagnates (static patterns / oscillators).
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

fn paint_background(
    life: &mut Life,
    ctx: &egui::Context,
    painter: &egui::Painter,
    rect: egui::Rect,
) {
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

    // Theme-aware, faintly blue-tinted. We paint over the panel's own fill (no base rect)
    // so the wallpaper tracks light/dark automatically.
    let dark = ctx.style().visuals.dark_mode;
    let (dot, cell_col) = if dark {
        (
            egui::Color32::from_rgb(42, 47, 62),
            egui::Color32::from_rgb(31, 34, 45),
        )
    } else {
        (
            egui::Color32::from_rgb(229, 233, 243),
            egui::Color32::from_rgb(240, 242, 249),
        )
    };

    // Live cells: barely-there rounded squares.
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
        ui.spacing_mut().interact_size.y = 44.0;
        ui.horizontal(|ui| {
            ui.heading("Kevin Guo");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button_text = match app.mobile_page {
                    MobilePage::About => "Notes",
                    MobilePage::Notes => "About",
                    MobilePage::Post => "All notes",
                };
                if ui.button(button_text).clicked() {
                    app.mobile_page = match app.mobile_page {
                        MobilePage::About | MobilePage::Post => MobilePage::Notes,
                        MobilePage::Notes => MobilePage::About,
                    };
                }
            });
        });
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.spacing_mut().interact_size.y = 44.0;
            ui.spacing_mut().item_spacing.y = 12.0;
            let scroll_id = match app.mobile_page {
                MobilePage::About => egui::Id::new("mobile_about"),
                MobilePage::Notes => egui::Id::new("mobile_notes"),
                MobilePage::Post => egui::Id::new(("mobile_post", app.current_post_index)),
            };
            egui::ScrollArea::vertical()
                .id_source(scroll_id)
                .auto_shrink([false, false])
                .show(ui, |ui| match app.mobile_page {
                    MobilePage::About => {
                        render_intro(ui);
                        ui.add_space(16.0);
                        render_links(ui);
                    }
                    MobilePage::Notes => {
                        ui.heading("Notes");
                        ui.add_space(8.0);
                        if show_blog_list(app, ui) {
                            app.mobile_page = MobilePage::Post;
                        }
                    }
                    MobilePage::Post => show_blog_content(app, ui),
                });
        });
}

// Re-applied every frame so it survives the Light/Dark toggle (which resets visuals).
fn apply_theme(ctx: &egui::Context) {
    let mut v = ctx.style().visuals.clone();
    if v.dark_mode {
        v.override_text_color = Some(egui::Color32::from_gray(205));
        v.selection.bg_fill = egui::Color32::from_rgb(45, 66, 104); // muted blue
    } else {
        // Darker body text — egui's default light gray reads as thin/faint on white.
        v.override_text_color = Some(egui::Color32::from_gray(45));
        v.selection.bg_fill = egui::Color32::from_rgb(189, 212, 243); // soft blue, readable
    }
    ctx.set_visuals(v);
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);
        let mobile = is_mobile_or_small_screen(ctx);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if mobile {
                ui.spacing_mut().interact_size.y = 44.0;
            }
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

        if !mobile {
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
        .min_width(640.0)
        .min_height(400.0)
}

fn make_frame_with_padding(padding: f32) -> egui::Frame {
    egui::Frame::none().inner_margin(egui::Margin::same(padding))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mobile_frame(
        ctx: &egui::Context,
        app: &mut TemplateApp,
        size: egui::Vec2,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ctx| mobile_layout(app, ctx),
        )
    }

    fn assert_text_fits(shape: &egui::Shape, width: f32, text_count: &mut usize) {
        match shape {
            egui::Shape::Text(text) if !text.galley.text().trim().is_empty() => {
                *text_count += 1;
                let bounds = text.visual_bounding_rect();
                assert!(
                    bounds.left() >= -1.0 && bounds.right() <= width + 1.0,
                    "text extends past a {width}px viewport: {:?} at {bounds:?}",
                    text.galley.text()
                );
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    assert_text_fits(shape, width, text_count);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn every_post_fits_phone_and_tablet_widths() {
        for width in [320.0, 375.0, 390.0, 430.0, 768.0, 1024.0] {
            let ctx = egui::Context::default();
            configure_style(&ctx);
            let mut app = TemplateApp {
                blog_posts: TemplateApp::load_blog_posts(),
                mobile_page: MobilePage::Post,
                ..Default::default()
            };
            for index in 0..app.blog_posts.len() {
                app.current_post_index = Some(index);
                // Render the entire article, including links near the end, without vertical culling.
                let output = mobile_frame(&ctx, &mut app, egui::vec2(width, 30000.0), vec![]);
                let mut text_count = 0;
                for shape in output.shapes {
                    assert_text_fits(&shape.shape, width, &mut text_count);
                }
                assert!(text_count > 0);
            }
        }
    }

    fn click_text(ctx: &egui::Context, app: &mut TemplateApp, text: &str) {
        let size = egui::vec2(375.0, 812.0);
        let output = mobile_frame(ctx, app, size, vec![]);
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(shape) if shape.galley.text() == text => {
                    Some(shape.visual_bounding_rect().center())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing visible control: {text}"));
        for pressed in [true, false] {
            mobile_frame(
                ctx,
                app,
                size,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
            );
        }
    }

    #[test]
    fn mobile_navigation_opens_a_post_and_returns_to_the_list() {
        let ctx = egui::Context::default();
        configure_style(&ctx);
        let mut app = TemplateApp {
            blog_posts: TemplateApp::load_blog_posts(),
            ..Default::default()
        };
        click_text(&ctx, &mut app, "Notes");
        assert!(matches!(app.mobile_page, MobilePage::Notes));
        click_text(&ctx, &mut app, "A layman's view of LLMs");
        assert!(matches!(app.mobile_page, MobilePage::Post));
        assert_eq!(app.current_post_index, Some(1));
        click_text(&ctx, &mut app, "All notes");
        assert!(matches!(app.mobile_page, MobilePage::Notes));
        click_text(&ctx, &mut app, "About");
        assert!(matches!(app.mobile_page, MobilePage::About));
    }

    #[test]
    fn mobile_article_scrolls_vertically() {
        let ctx = egui::Context::default();
        configure_style(&ctx);
        let mut app = TemplateApp {
            blog_posts: TemplateApp::load_blog_posts(),
            current_post_index: Some(0),
            mobile_page: MobilePage::Post,
            ..Default::default()
        };
        let size = egui::vec2(375.0, 812.0);
        mobile_frame(&ctx, &mut app, size, vec![]);
        mobile_frame(
            &ctx,
            &mut app,
            size,
            vec![
                egui::Event::PointerMoved(egui::pos2(150.0, 400.0)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -600.0),
                    modifiers: egui::Modifiers::default(),
                },
            ],
        );
        let output = mobile_frame(&ctx, &mut app, size, vec![]);
        let visible_text: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text)
                    if shape.clip_rect.intersects(text.visual_bounding_rect()) =>
                {
                    Some(text.galley.text())
                }
                _ => None,
            })
            .collect();
        assert!(!visible_text.contains(&"Notes after Ramp"));
        assert!(visible_text.contains(&"All notes"));
    }
}
