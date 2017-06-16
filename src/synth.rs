
use voice;
use voice::*;

pub struct Synth {
    pub voice: voice::Voice,
}

impl Synth {
    pub fn new() -> Synth {
        Synth { voice: voice::Voice::new() }
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.voice.set_fs(fs);
    }
    pub fn noteon(&mut self, f0: f32, vel: f32) {
        self.voice.on = true;
        self.voice.f0 = f0;
        self.voice.vel = vel;
        self.voice.initialize();
    }
    pub fn noteoff(&mut self) {
        // self.voice.on = false;
        self.voice.noteoff();
    }
    pub fn get_amp(&mut self) -> f32 {
        self.voice.get_amp()
    }
    pub fn cleanup(&mut self) {
        self.voice.cleanup();
    }
}
