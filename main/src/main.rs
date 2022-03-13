use audio_host::AudioHost;
use eframe::egui::vec2;
use filters::Vcf;
use gui::ModularSynth;
use module::Module;
use oscillators::{lfo::Lfo, vco::Vco};
use rack::Rack;
use utility_modules::{amplifier::Vca, clock::Clock, envelope::Adsr};

fn main() -> anyhow::Result<()> {
    let window_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(1000.0, 515.0)),
        ..Default::default()
    };

    let clock = Clock::default();
    let lfo = Lfo::default();
    let vco = Vco::default();
    let adsr = Adsr::default();
    let vca = Vca::default();
    let vcf = Vcf::default();

    let mut rack = Rack::new();
    let clock_handle = rack.add_module(&clock);
    let lfo_handle = rack.add_module(&lfo);
    let vco_handle = rack.add_module(&vco);
    let adsr_handle = rack.add_module(&adsr);
    let vca_handle = rack.add_module(&vca);
    let vcf_handle = rack.add_module(&vcf);

    let panels = vec![
        (clock_handle, clock.create_panel()),
        (lfo_handle, lfo.create_panel()),
        (vco_handle, vco.create_panel()),
        (adsr_handle, adsr.create_panel()),
        (vca_handle, vca.create_panel()),
        (vcf_handle, vcf.create_panel()),
    ];

    let mut audio_host = AudioHost::default();
    audio_host.start(rack)?;
    let app = ModularSynth::new(audio_host, panels);
    eframe::run_native(Box::new(app), window_options);
}
