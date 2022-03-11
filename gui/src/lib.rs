use audio_host::AudioHost;
use eframe::{egui, epi};
use module::{ModuleHandle, Panel};

mod connections;
mod fonts;
mod panels;

use crate::connections::Connections;

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
        fonts::configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let cmd_shift = egui::Modifiers {
            command: true,
            shift: true,
            ..Default::default()
        };
        if ctx.input_mut().consume_key(cmd_shift, egui::Key::D) {
            ctx.set_debug_on_hover(!ctx.debug_on_hover());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (handle, panel) in &mut self.panels {
                    ui.add(panels::panel_to_widget(handle, panel.as_mut()));
                }
            });
            self.connections.update(&self.audio_host, ui);
        });
    }
}
