use oscillator::*;

pub trait IsVoice {
    fn new() -> Voice;
    fn set_fs(&mut self, f64);
    fn get_amp(&mut self) -> f32;
    fn initialize(&mut self);
    fn cleanup(&mut self);
    // fn reset(&mut self);
    // fn getAmp(&self) -> f32;
}

pub struct Voice {
    pub f0: f32,
    pub vel: f32,
    pub on: bool,
    pub osc1: OscMulti,
}

impl IsVoice for Voice {
    fn new() -> Voice {
        Voice {
            f0: 0f32,
            vel: 0f32,
            on: false,
            osc1: OscMulti::new(),
        }
    }
    fn set_fs(&mut self, fs: f64) {
        self.osc1.set_fs(fs);
    }
    fn get_amp(&mut self) -> f32 {
        if self.on {
            self.vel * self.osc1.get_amp()
        } else {
            0.0
        }

    }
    fn initialize(&mut self) {
        self.osc1.reset(self.f0);
    }
    fn cleanup(&mut self) {
        self.osc1.cleanup();
    }
    // self.osc1.reset(&mut self) {
    //     self.osc1.set_f0 = self.f0;
    //     self.osc1.reset_phase();
    // }
}
