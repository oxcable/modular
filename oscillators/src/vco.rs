use egui::{Align, Layout};
use eurorack::{Voltage, AUDIO_VOLTS, V_OCT_F0};
use gui::jack::Jack;
use module::{AudioUnit, Module, Panel};

#[derive(Default)]
pub struct Vco;

impl Module for Vco {
    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        3
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit + Send> {
        Box::new(VcoUnit::new())
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(VcoPanel {})
    }
}

pub struct VcoUnit {
    phase: f32,
    phase_delta: f32,
    last_tri: f32,
}

impl rack::ModuleIO for VcoUnit {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 3;
}

impl VcoUnit {
    pub const V_OCT_IN: usize = 0;

    pub const SAW_OUT: usize = 0;
    pub const SQUARE_OUT: usize = 1;
    pub const TRI_OUT: usize = 2;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        VcoUnit {
            phase: 0.0,
            phase_delta: 0.0,
            last_tri: 0.0,
        }
    }
}

impl AudioUnit for VcoUnit {
    fn reset(&mut self, sample_rate: usize) {
        self.phase_delta = V_OCT_F0 / sample_rate as f32;
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        // Compute the new waveform phase using the input voltage.
        let v_oct = inputs[Self::V_OCT_IN].unwrap_or(0.0);
        let dt = self.phase_delta * 2f32.powf(v_oct);
        self.phase = (self.phase + dt) % 1.0;

        // Directly compute antialiased saw.
        outputs[Self::SAW_OUT] = AUDIO_VOLTS * (2.0 * self.phase - 1.0 - poly_blep(self.phase, dt));

        // Piecewise compute antialiased square.
        let raw_sq = if self.phase > 0.5 { 1.0 } else { -1.0 };
        let aa_sq = raw_sq - poly_blep(self.phase, dt) + poly_blep((self.phase + 0.5) % 1.0, dt);
        outputs[Self::SQUARE_OUT] = AUDIO_VOLTS * aa_sq;

        // Compute triangle as integration of square.
        self.last_tri = 2.0 * dt * aa_sq + (1.0 - 2.0 * dt) * self.last_tri;
        outputs[Self::TRI_OUT] = AUDIO_VOLTS * self.last_tri;
    }
}

pub struct VcoPanel {}

impl Panel for VcoPanel {
    fn width(&self) -> usize {
        5
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("VCO");
        ui.add_space(20.0);
        ui.add(Jack::input(handle.input(VcoUnit::V_OCT_IN)));
        ui.label("V/Oct");
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add(Jack::output(handle.output(VcoUnit::TRI_OUT)));
            ui.label("Tri");
            ui.add(Jack::output(handle.output(VcoUnit::SQUARE_OUT)));
            ui.label("Square");
            ui.add(Jack::output(handle.output(VcoUnit::SAW_OUT)));
            ui.label("Saw");
        });
    }
}

/// Computes a single offset for PolyBLEP antialiasing.
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        // t ~= 0
        let t = t / dt;
        -t * t + 2.0 * t - 1.0
    } else if t > 1.0 - dt {
        // t ~= 1
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}
