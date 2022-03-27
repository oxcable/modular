use std::{collections::HashMap, fs::File, hash::Hash, path::Path};

use ::widgets::jack::JackInteraction;
use audio_host::{AudioHost, AudioMessage};
use eframe::egui::*;
use eframe::epaint::QuadraticBezierShape;
use module::{registry::ModuleRegistry, ModuleHandle, ModuleInput, ModuleOutput, Panel};

use crate::panels;

pub(crate) struct Patch {
    modules: Vec<ModuleInstance>,
    connections: Vec<Connection>,
}

impl Patch {
    pub(crate) fn new() -> Self {
        Patch {
            modules: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub(crate) fn add_module(
        &mut self,
        registry: &mut ModuleRegistry,
        audio_host: &AudioHost,
        id: String,
    ) -> ModuleHandle {
        let (handle, module) = registry.create_module(&id).unwrap();
        audio_host.send_message(AudioMessage::AddModule(
            handle,
            module.inputs(),
            module.outputs(),
            module.create_audio_unit(),
        ));
        self.modules.push(ModuleInstance {
            id,
            handle,
            panel: module.create_panel(),
        });
        handle
    }

    pub(crate) fn save<P: AsRef<Path>>(&self, path: P) {
        let mut handle_indices: HashMap<ModuleHandle, usize> = self
            .modules
            .iter()
            .enumerate()
            .map(|(i, inst)| (inst.handle, i))
            .collect();
        handle_indices.insert(rack::AUDIO_OUTPUT_HANDLE, rack::AUDIO_OUTPUT_HANDLE.0);

        let mut serialized = SerializedPatch::default();
        for module in &self.modules {
            serialized.modules.push(module.id.clone());
        }
        for connection in &self.connections {
            serialized.connections.push(SerializedConnection {
                src_index: handle_indices[&connection.output.module],
                src_channel: connection.output.channel,
                dst_index: handle_indices[&connection.input.module],
                dst_channel: connection.input.channel,
            });
        }

        let file = File::create(path).unwrap();
        serde_json::to_writer_pretty(file, &serialized).unwrap();
    }

    pub(crate) fn update(&mut self, host: &AudioHost, ui: &mut Ui) {
        // Draw panels.
        ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for module in &mut self.modules {
                    ui.add(panels::panel_to_widget(
                        &module.handle,
                        module.panel.as_mut(),
                    ));
                }
                // Always add audio output as the last panel.
                ui.add(panels::panel_to_widget(
                    &rack::AUDIO_OUTPUT_HANDLE,
                    &mut panels::AudioOutputPanel,
                ));
            });
        });

        // Handle any interactions from Jack widgets:
        let mut pending_source = None;
        if let Some(interaction) = JackInteraction::get(ui) {
            match interaction {
                JackInteraction::PendingInput(input) => pending_source = locate(ui, input),
                JackInteraction::PendingOutput(output) => pending_source = locate(ui, output),
                JackInteraction::CreateConnection(output, input) => {
                    self.maybe_clear_input(input, host);
                    host.send_message(AudioMessage::ConnectModules(output, input));
                    self.connections.push(Connection { output, input });
                    JackInteraction::clear(ui);
                }
                JackInteraction::ClearInput(input) => {
                    self.maybe_clear_input(input, host);
                    JackInteraction::clear(ui);
                }
                JackInteraction::ClearOutput(output) => {
                    self.clear_all_outputs(output, host);
                    JackInteraction::clear(ui);
                }
            }
        }

        // Draw existing connections:
        for c in &self.connections {
            Cable::new(locate(ui, c.output).unwrap(), locate(ui, c.input).unwrap()).draw(ui);
        }

        // Handle ongoing new connection:
        if let Some(src_pos) = pending_source {
            // Use <esc> or right click to cancel new connection.
            if ui.ctx().input().key_pressed(Key::Escape)
                || ui.input().pointer.button_down(PointerButton::Secondary)
            {
                JackInteraction::clear(ui);
            } else {
                let hover_pos = ui.ctx().input().pointer.hover_pos();
                if let Some(pos) = hover_pos {
                    Cable::new(src_pos, pos).draw(ui);
                }
            }
        }
    }

    fn maybe_clear_input(&mut self, input: ModuleInput, host: &AudioHost) {
        if let Some(i) = self.connections.iter().position(|c| c.input == input) {
            host.send_message(AudioMessage::DisconnectModules(
                self.connections[i].output,
                input,
            ));
            self.connections.swap_remove(i);
        }
    }

    fn clear_all_outputs(&mut self, output: ModuleOutput, host: &AudioHost) {
        while let Some(i) = self.connections.iter().position(|c| c.output == output) {
            host.send_message(AudioMessage::DisconnectModules(
                output,
                self.connections[i].input,
            ));
            self.connections.swap_remove(i);
        }
    }
}

struct Cable {
    src: Pos2,
    dst: Pos2,
}

impl Cable {
    fn new(src: Pos2, dst: Pos2) -> Self {
        Cable { src, dst }
    }

    fn draw(self, ui: &mut Ui) {
        let stroke = if ui.visuals().dark_mode {
            Stroke::new(5.0, Color32::from_white_alpha(64))
        } else {
            Stroke::new(5.0, Color32::from_black_alpha(128))
        };

        let dy = (self.src.y - self.dst.y).abs();
        let sag = vec2(0.0, 0.5 * dy + 30.0);
        let midpoint = self.src + 0.5 * (self.dst - self.src) + sag;

        ui.painter()
            .add(Shape::from(QuadraticBezierShape::from_points_stroke(
                [self.src, midpoint, self.dst],
                false,
                Color32::default(),
                stroke,
            )));
    }
}

struct ModuleInstance {
    id: String,
    handle: ModuleHandle,
    panel: Box<dyn Panel>,
}

#[derive(Copy, Clone, Debug)]
struct Connection {
    output: ModuleOutput,
    input: ModuleInput,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct SerializedPatch {
    modules: Vec<String>,
    connections: Vec<SerializedConnection>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerializedConnection {
    src_index: usize,
    src_channel: usize,
    dst_index: usize,
    dst_channel: usize,
}

fn locate<T>(ui: &Ui, io: T) -> Option<Pos2>
where
    T: Hash,
{
    let id = Id::new(io);
    ui.memory().data.get_temp(id)
}
