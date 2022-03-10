use audio_host::AudioHost;
use eframe::egui::vec2;
use gui::ModularSynth;
use module::Module;
use oscillators::{
    lfo::{Lfo, LfoUnit},
    vco::{Vco, VcoUnit},
};
use rack::Rack;
use utility_modules::amplifier::{Vca, VcaUnit};

fn main() -> anyhow::Result<()> {
    let window_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(1000.0, 515.0)),
        ..Default::default()
    };

    let lfo = Lfo::default();
    let vco = Vco::default();
    let vca = Vca::default();

    let mut rack = Rack::new();
    let lfo_handle = rack.add_module(&lfo);
    let vco_handle = rack.add_module(&vco);
    let vca_handle = rack.add_module(&vca);
    rack.connect(
        lfo_handle.output(LfoUnit::TRI_OUT),
        vca_handle.input(VcaUnit::CV_IN),
    )?;
    rack.connect(
        vco_handle.output(VcoUnit::TRI_OUT),
        vca_handle.input(VcaUnit::AUDIO_IN),
    )?;
    rack.connect(vca_handle.output(VcaUnit::AUDIO_OUT), Rack::audio_output())?;
    let mut audio_host = AudioHost::default();

    let panels = vec![
        (lfo_handle, lfo.create_panel()),
        (vco_handle, vco.create_panel()),
        (vca_handle, vca.create_panel()),
    ];
    let app = ModularSynth::new(panels);

    audio_host.start(rack)?;
    eframe::run_native(Box::new(app), window_options);
}
