use oscillator::*;
use adsr::*;
use observer;
use observer::*;
use midi;
use midi::*;

// pub trait IsVoice {
//     fn new() -> Voice;
//     fn set_fs(&mut self, f64);
//     fn get_amp(&mut self) -> f32;
//     fn initialize(&mut self);
//     fn cleanup(&mut self);
//     // fn noteoff(&mut self);
// }

pub struct Voice {
    pub f0: f32,
    pub vel: f32,
    pub on: bool,
    pub osc1: OscBLIT,
    pub adsr: ADSR,
}

impl Voice {
    pub fn new() -> Voice {
        Voice {
            f0: 0f32,
            vel: 0f32,
            on: false,
            osc1: OscBLIT::new(),
            adsr: ADSR::new(),
        }
    }
    // fn set_fs(&mut self, fs: f64) {
    //     self.adsr.initialize(fs);
    //     self.osc1.set_fs(fs);
    // }
    // fn get_amp(&mut self) -> f32 {
    //     if self.on {
    //         self.adsr.step();
    //         self.adsr.amp * self.vel * self.osc1.get_amp()
    //     } else {
    //         match self.adsr.state {
    //             ADSRSTATE::Release => {
    //                 self.adsr.pa.reset(self.adsr.f0);
    //                 self.adsr.step();
    //                 self.adsr.amp * self.vel * self.osc1.get_amp()
    //             }
    //             _ => {
    //                 0.0
    //             }
    //         }
    //     }

    // }
    fn initialize(&mut self) {
        self.adsr.reset();
        self.osc1.reset(self.f0 as f64);
    }
    // fn cleanup(&mut self) {
    //     self.osc1.cleanup();
    // }
    // fn noteoff(&mut self) {
    //     self.on = false;
    //     self.adsr.state = ADSRSTATE::Release;
    // }
    // self.osc1.reset(&mut self) {
    //     self.osc1.set_f0 = self.f0;
    //     self.osc1.reset_phase();
    // }
}

impl Observer for Voice {
    fn next(&mut self, mm: MidiMessage) {}
    fn next_noteon(&mut self, f0: f32, vel: f32) {
        self.on = true;
        self.f0 = f0;
        self.vel = vel;
        self.initialize();
    }
    fn next_noteoff(&mut self) {
        self.on = false;
        self.adsr.state = ADSRSTATE::Release;
    }
    fn next_fs(&mut self, fs: f64) {
        self.adsr.initialize(fs);
        self.osc1.set_fs(fs);
    }
    fn next_blit(&mut self, blit: bool) {}
    fn next_postfilter(&mut self, pf: bool) {}
    fn get_amp(&mut self) -> f32 {
        if self.on {
            self.adsr.step();
            self.adsr.amp * self.vel * self.osc1.get_amp()
        } else {
            match self.adsr.state {
                ADSRSTATE::Release => {
                    self.adsr.pa.reset(self.adsr.f0);
                    self.adsr.step();
                    self.adsr.amp * self.vel * self.osc1.get_amp()
                }
                _ => {
                    0.0
                }
            }
        }
    }
    fn cleanup(&mut self) {
        self.osc1.cleanup();
    }
}

