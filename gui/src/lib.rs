use eframe::{egui, epi};

const HP_PIXELS: usize = 20;
const PANEL_HEIGHT: usize = 25 * HP_PIXELS;

pub trait Panel {
    fn width(&self) -> usize;
    fn update(&mut self, ui: &mut egui::Ui);
}

pub struct ModularSynth {
    panels: Vec<Box<dyn Panel>>,
}

impl ModularSynth {
    pub fn new(panels: Vec<Box<dyn Panel>>) -> Self {
        ModularSynth { panels }
    }
}

impl epi::App for ModularSynth {
    fn name(&self) -> &str {
        "oxcable :: Modular Synthesizer"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for panel in &mut self.panels {
                    let width = HP_PIXELS * panel.width();
                    let desired_size = egui::vec2(width as f32, PANEL_HEIGHT as f32);
                    let (rect, _response) =
                        ui.allocate_exact_size(desired_size, egui::Sense::hover());

                    if ui.is_rect_visible(rect) {
                        ui.painter().rect(
                            rect,
                            10.0,
                            ui.visuals().faint_bg_color,
                            ui.visuals().noninteractive().bg_stroke,
                        );
                        let mut panel_ui = ui.child_ui(
                            rect.shrink(10.0),
                            egui::Layout::top_down(egui::Align::Center),
                        );
                        panel.update(&mut panel_ui);
                    }
                }
            });
        });
    }
}
