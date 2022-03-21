use audio_host::AudioHost;
use eframe::{egui, epi};
use module::{registry::ModuleRegistry, ModuleHandle, Panel};

mod connections;
mod fonts;
mod panels;

use crate::connections::Connections;

pub struct ModularSynth {
    registry: ModuleRegistry,
    panels: Vec<(ModuleHandle, Box<dyn Panel>)>,
    audio_host: AudioHost,
    connections: Connections,
}

impl ModularSynth {
    pub fn new(
        registry: ModuleRegistry,
        audio_host: AudioHost,
        mut panels: Vec<(ModuleHandle, Box<dyn Panel>)>,
    ) -> Self {
        panels.push((
            rack::AUDIO_OUTPUT_HANDLE,
            Box::new(panels::AudioOutputPanel),
        ));
        ModularSynth {
            registry,
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
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Modules", |ui| {
                    let mut modules = self.registry.all_modules();
                    modules.sort();
                    for module in modules.into_iter() {
                        if ui.button(&module).clicked() {
                            println!("Add module: {}", module);
                            ui.close_menu();
                        }
                    }
                });
                ui.menu_button("Debug", |ui| {
                    if ui.button("Toggle layout on hover").clicked() {
                        ctx.set_debug_on_hover(!ctx.debug_on_hover());
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (handle, panel) in &mut self.panels {
                        ui.add(panels::panel_to_widget(handle, panel.as_mut()));
                    }
                });
                self.connections.update(&self.audio_host, ui);
            });
        });
    }
}
