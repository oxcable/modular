use audio_host::AudioHost;
use eframe::egui::vec2;
use gui::ModularSynth;
use modules::builtin_modules;
use rack::Rack;

fn main() -> anyhow::Result<()> {
    let window_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(875.0, 540.0)),
        ..Default::default()
    };

    let mut audio_host = AudioHost::default();
    audio_host.start(Rack::new())?;
    let app = ModularSynth::new(builtin_modules(), audio_host);
    eframe::run_native(Box::new(app), window_options);
}
