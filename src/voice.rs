use oscillator::*;
use adsr::*;
use observer;
use observer::*;
use midi;
use midi::*;
use types;

// pub trait IsVoice {
//     fn new() -> Voice;
//     fn set_fs(&mut self, f64);
//     fn get_amp(&mut self) -> f32;
//     fn initialize(&mut self);
//     fn cleanup(&mut self);
//     // fn noteoff(&mut self);
// }

pub struct Voice<'a> {
    // pub f0: f32,
    pub vel: f32,
    // pub on: bool,
    // pub osc1: OscBLIT,
    // pub adsr: ADSR,
    pub f0: Observable<'a, types::f0>,
}

impl<'a> Voice<'a> {
    pub fn new() -> Voice<'a> {
        Voice {
            // f0: 0f32,
            vel: 0f32,
            // on: false,
            // osc1: OscBLIT::new(),
            // adsr: ADSR::new(),
            f0: Observable::new(types::f0(0f32)),
        }
    }
    // pub fn connect() {
    //     self.midi_message_processor.noteon[0].observers.push(&mut *voice);
    //     self.midi_message_processor.noteoff[0].observers.push(&mut *voice);
    //     self.midi_message_processor.noteon[1].observers.push(&mut *voice2);
    //     self.midi_message_processor.noteoff[1].observers.push(&mut *voice2);
    //     self.midi_message_processor.noteon[2].observers.push(&mut *voice3);
    //     self.midi_message_processor.noteoff[2].observers.push(&mut *voice3);
    //     self.midi_message_processor.noteon[3].observers.push(&mut *voice4);
    //     self.midi_message_processor.noteoff[3].observers.push(&mut *voice4);
    // }

}

impl<'a> Observer<types::noteon> for Voice<'a> {
    fn next(&mut self, no: types::noteon) {
        println!(" VOICE RECEIVED NOTEON: {}", no.0 as f32);
        self.f0.update(types::f0(no.0));
        // self.f1.update(types::f0(2.04f32 * no.0));
        self.vel = no.1
    }
}
impl<'a> Observer<types::noteoff> for Voice<'a> {
    fn next(&mut self, no: types::noteoff) {
        self.vel = 0f32
    }
}



// impl Voice {
//     pub fn new() -> Voice {
//         Voice {
//             f0: 0f32,
//             vel: 0f32,
//             on: false,
//             osc1: OscBLIT::new(),
//             adsr: ADSR::new(),
//         }
//     }
//     // fn set_fs(&mut self, fs: types::fs) {
//     //     self.adsr.initialize(fs);
//     //     self.osc1.set_fs(fs);
//     // }
//     pub fn get_amp(&mut self) -> f32 {
//         if self.on {
//             self.adsr.step();
//             self.adsr.amp * self.vel * self.osc1.get_amp()
//         } else {
//             match self.adsr.state {
//                 ADSRSTATE::Release => {
//                     self.adsr.pa.reset(self.adsr.f0);
//                     self.adsr.step();
//                     self.adsr.amp * self.vel * self.osc1.get_amp()
//                 }
//                 _ => {
//                     0.0
//                 }
//             }
//         }

//     }
//     fn initialize(&mut self) {
//         self.adsr.reset();
//         self.osc1.reset(self.f0 as f64);
//     }
//     pub fn cleanup(&mut self) {
//         self.osc1.cleanup();
//     }
//     // fn noteoff(&mut self) {
//     //     self.on = false;
//     //     self.adsr.state = ADSRSTATE::Release;
//     // }
//     // self.osc1.reset(&mut self) {
//     //     self.osc1.set_f0 = self.f0;
//     //     self.osc1.reset_phase();
//     // }
// }

// impl Observer<(f32, f32)> for Voice {
//     fn next(&mut self, f0vel: (f32, f32)) {
//         self.on = true;
//         self.f0 = f0vel.0;
//         self.vel = f0vel.1;
//         self.initialize();
//     }
// }

// impl Observer<types::fs> for Voice {
//     fn next(&mut self, fs: types::fs) {
//         self.adsr.initialize(fs);
//         self.osc1.set_fs(fs);
//     }
// }

// impl Observer<types::noteoff> for Voice {
//     fn next(&mut self, fs: types::noteoff) {
//         self.on = false;
//         self.adsr.state = ADSRSTATE::Release;
//     }
// }

//     // fn next_noteon(&mut self, f0: f32, vel: f32) {
//     //     self.on = true;
//     //     self.f0 = f0;
//     //     self.vel = vel;
//     //     self.initialize();
//     // }
//     // fn next_noteoff(&mut self) {
//     //     self.on = false;
//     //     self.adsr.state = ADSRSTATE::Release;
//     // }
//     // fn next_fs(&mut self, fs: types::fs) {
//     //     self.adsr.initialize(fs);
//     //     self.osc1.set_fs(fs);
//     // }
//     // fn next_blit(&mut self, blit: bool) {}
//     // fn next_postfilter(&mut self, pf: bool) {}
//     // fn get_amp(&mut self) -> f32 {
//     //     if self.on {
//     //         self.adsr.step();
//     //         self.adsr.amp * self.vel * self.osc1.get_amp()
//     //     } else {
//     //         match self.adsr.state {
//     //             ADSRSTATE::Release => {
//     //                 self.adsr.pa.reset(self.adsr.f0);
//     //                 self.adsr.step();
//     //                 self.adsr.amp * self.vel * self.osc1.get_amp()
//     //             }
//     //             _ => {
//     //                 0.0
//     //             }
//     //         }
//     //     }
//     // }
//     // fn cleanup(&mut self) {
//     //     self.osc1.cleanup();
//     // }
// // }


