use audio_host::AudioHost;
use eframe::egui::vec2;
use gui::ModularSynth;
use module::Module;
use oscillators::{
    lfo::Lfo,
    vco::{Vco, VcoUnit},
};
use rack::Rack;

fn main() -> anyhow::Result<()> {
    let window_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(1000.0, 515.0)),
        ..Default::default()
    };

    let lfo = Lfo::default();
    let vco = Vco::default();

    let mut rack = Rack::new();
    let lfo_handle = rack.add_module(&lfo);
    let vco_handle = rack.add_module(&vco);
    rack.connect(vco_handle.output(VcoUnit::TRI_OUT), Rack::audio_output())?;
    let mut audio_host = AudioHost::default();

    let panels = vec![
        (lfo_handle, lfo.create_panel()),
        (vco_handle, vco.create_panel()),
    ];
    let app = ModularSynth::new(panels);

    audio_host.start(rack)?;
    eframe::run_native(Box::new(app), window_options);
}
