use audio_host::AudioHost;
use eframe::{egui, epi};
use module::{ModuleHandle, Panel};

pub mod jack;
pub mod knob;

mod connections;
mod panels;

use connections::Connections;

const HP_PIXELS: usize = 20;
const PANEL_HEIGHT: usize = 25 * HP_PIXELS;

pub struct ModularSynth {
    panels: Vec<(ModuleHandle, Box<dyn Panel>)>,
    audio_host: AudioHost,
    connections: Connections,
}

impl ModularSynth {
    pub fn new(audio_host: AudioHost, mut panels: Vec<(ModuleHandle, Box<dyn Panel>)>) -> Self {
        panels.push((
            rack::AUDIO_OUTPUT_HANDLE,
            Box::new(panels::AudioOutputPanel),
        ));
        ModularSynth {
            audio_host,
            panels,
            connections: Connections::new(),
        }
    }
}

impl epi::App for ModularSynth {
    fn name(&self) -> &str {
        "oxcable :: Modular Synthesizer"
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        use egui::{FontData, FontDefinitions, FontFamily, FontId, Style, TextStyle};

        // Load font data:
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "SpaceMonoBoldItalic".to_owned(),
            FontData::from_static(include_bytes!("../../assets/SpaceMono-BoldItalic.ttf")),
        );
        fonts.font_data.insert(
            "SpaceGrotesk".to_owned(),
            FontData::from_static(include_bytes!("../../assets/SpaceGrotesk-Medium.ttf")),
        );

        // Define font families:
        let title = FontFamily::Name("title".into());
        let regular = FontFamily::Name("regular".into());
        fonts
            .families
            .insert(title.clone(), vec!["SpaceMonoBoldItalic".to_owned()]);
        fonts
            .families
            .insert(regular.clone(), vec!["SpaceGrotesk".to_owned()]);
        ctx.set_fonts(fonts);

        // Configure default fonts:
        let mut style: Style = (*ctx.style()).clone();
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(30.0, title));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(18.0, regular.clone()));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::new(14.0, regular));
        ctx.set_style(style);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (handle, panel) in &mut self.panels {
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
                        panel.update(handle, &mut panel_ui);
                    }
                }
            });
            self.connections.update(&self.audio_host, ui);
        });
    }
}
