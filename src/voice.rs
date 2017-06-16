use oscillator::*;

pub trait IsVoice {
    fn new() -> Voice;
    fn set_fs(&mut self, f64);
    fn get_amp(&mut self) -> f32;
    fn initialize(&mut self);
    fn cleanup(&mut self);
    fn noteoff(&mut self);
}

pub enum ADSRSTATE {
    Attack,
    Decay,
    Sustain,
    Fade,
    Release,
    Off
}
pub struct ADSR {
    pub amp: f32,
    pub pa: PhaseAccumulator,
    pub state: ADSRSTATE,
    pub f0: f64,
}

impl ADSR {
    pub fn new() -> ADSR {
        ADSR{pa: PhaseAccumulator::new(), amp: 0f32, state: ADSRSTATE::Attack, f0: 0.5f64}
    }
    pub fn initialize(&mut self, fs: f64) {
        self.pa.set_fs(fs);
        // f0=1, i.e. 1 cycle per second
        self.pa.reset(self.f0);
    }
    pub fn step(&mut self) {
        self.pa.step();

        let attack = 0.2;
        let decay = 0.2;
        let sustain = 0.2;
        let sustain_vel = 0.5;
        let fade = 1.0 - (attack + decay + sustain);
        let release = 0.2;
        // let i range from 0 to 1.
        let mut i = ((self.pa.normalize_index()+1f64)/2f64) as f32;
        match self.state {
            ADSRSTATE::Attack => {
                let i = i/attack;
                if i <= 1.0 {
                    self.amp = i;
                } else {
                    self.state = ADSRSTATE::Decay;
                }
            }
            _ => {}
        }
        match self.state {
            ADSRSTATE::Decay => {
                i = (i - attack)/decay;
                // The first a's might actually be larger than the last
                // attack value. No problem.
                let a = 1.0 - i*(1.0-sustain_vel);
                if a >= sustain_vel {
                    self.amp = a;
                } else {
                    self.state = ADSRSTATE::Sustain;
                }
                return;
            }
            _ => {}
        }
        match self.state {
            ADSRSTATE::Sustain => {
                i = (i - attack - decay)/sustain;
                if i <= 1.0 {
                    return;
                } else {
                    self.state = ADSRSTATE::Fade;
                }
                return;
            }
            _ => {}            
        }
        match self.state {
            ADSRSTATE::Fade => {
                i = (i - attack - decay - sustain)/fade;
                let a = sustain_vel * (1.0 - i);
                if i < 0.99 {
                    self.amp = a;
                } else {
                    self.amp = 0.0;
                    self.state = ADSRSTATE::Off;
                }
                return;
            }
            _ => {}
        }
        match self.state {
            ADSRSTATE::Release => {
                let i = i/release;
                if i <= 1.0 {
                    self.amp = (1.0-i)*self.amp;
                } else {
                    self.state = ADSRSTATE::Off;
                }
            }
            _ => {
            }
        }
        match self.state {
            ADSRSTATE::Off => {}
            _ => {}
        }
    }
    pub fn reset(&mut self) {
        self.amp = 0.0;
        self.pa.reset(self.f0);
        self.state = ADSRSTATE::Attack;
    }
}

pub struct Voice {
    pub f0: f32,
    pub vel: f32,
    pub on: bool,
    pub osc1: OscBLIT,
    pub adsr: ADSR,
}

impl IsVoice for Voice {
    fn new() -> Voice {
        Voice {
            f0: 0f32,
            vel: 0f32,
            on: false,
            osc1: OscBLIT::new(),
            adsr: ADSR::new(),
        }
    }
    fn set_fs(&mut self, fs: f64) {
        self.adsr.initialize(fs);
        self.osc1.set_fs(fs);
    }
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
    fn initialize(&mut self) {
        self.adsr.reset();
        self.osc1.reset(self.f0 as f64);
    }
    fn cleanup(&mut self) {
        self.osc1.cleanup();
    }
    fn noteoff(&mut self) {
        self.on = false;
        self.adsr.state = ADSRSTATE::Release;
    }
    // self.osc1.reset(&mut self) {
    //     self.osc1.set_f0 = self.f0;
    //     self.osc1.reset_phase();
    // }
}
