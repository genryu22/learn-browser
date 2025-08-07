use eframe::egui::{self, UiBuilder};
use learn_browser::url::{Url, request, strip_html_tags};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };
    eframe::run_native(
        "Browser Window",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "my_font".to_owned(),
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                    "../NotoSansJP-Regular.ttf"
                ))),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "my_font".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::<BrowserApp>::default())
        }),
    )
}

struct BrowserApp {
    text_content: String,
    error_message: Option<String>,
    scroll_offset: f32,
}

impl Default for BrowserApp {
    fn default() -> Self {
        let mut app = Self {
            text_content: String::new(),
            error_message: None,
            scroll_offset: 0.0,
        };
        app.fetch_content();
        app
    }
}

impl BrowserApp {
    fn fetch_content(&mut self) {
        self.error_message = None;

        match Url::new("https://browser.engineering/examples/xiyouji.html") {
            Ok(url) => match request(&url) {
                Ok(response) => {
                    let clean_text = strip_html_tags(&response.body);
                    self.text_content = clean_text;
                }
                Err(e) => {
                    self.error_message = Some(format!("Request failed: {}", e));
                }
            },
            Err(e) => {
                self.error_message = Some(format!("URL parsing failed: {}", e));
            }
        }
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.scroll_offset += 100.0;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.scroll_offset = (self.scroll_offset - 100.0).max(0.0);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            } else {
                let hstep = 13.0;
                let vstep = 18.0;

                let mut x = 0.;
                let mut y = 0.;

                for ch in self.text_content.chars() {
                    if y + vstep >= self.scroll_offset && y <= HEIGHT + self.scroll_offset {
                        let pos = egui::pos2(x, y - self.scroll_offset);
                        ui.scope_builder(
                            UiBuilder::new()
                                .max_rect(egui::Rect::from_min_size(pos, egui::vec2(hstep, vstep))),
                            |ui| {
                                ui.label(ch.to_string());
                            },
                        );
                    }

                    x += hstep;
                    if x >= WIDTH - hstep {
                        y += vstep;
                        x = hstep;
                    }
                }
            }
        });
    }
}
