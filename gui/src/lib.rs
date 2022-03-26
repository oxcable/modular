use audio_host::{AudioHost, AudioMessage};
use eframe::{egui, epi};
use module::registry::ModuleRegistry;

mod fonts;
mod panels;
mod patch;

use crate::patch::Patch;

pub struct ModularSynth {
    registry: ModuleRegistry,
    audio_host: AudioHost,
    patch: Patch,
}

impl ModularSynth {
    pub fn new(registry: ModuleRegistry, audio_host: AudioHost) -> Self {
        ModularSynth {
            registry,
            audio_host,
            patch: Patch::new(),
        }
    }

    fn add_module(&mut self, id: String) {
        let (handle, module) = self.registry.create_module(&id).unwrap();
        self.audio_host.send_message(AudioMessage::AddModule(
            handle,
            module.inputs(),
            module.outputs(),
            module.create_audio_unit(),
        ));
        self.patch.add_module(id, handle, module);
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
                    modules.sort_by_key(|m| m.name.clone());
                    for manifest in modules.into_iter() {
                        if ui.button(manifest.name).clicked() {
                            self.add_module(manifest.id);
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
            self.patch.update(&self.audio_host, ui);
        });
    }
}
