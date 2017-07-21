use types;
use std::collections::VecDeque;
use observer;
use observer::*;
use midi;
use midi::*;
use std::io;
use std::io::Write;
use oscillator::*;
use voice;

const NOSC: usize = 11;

pub struct Polyphony<'a> {
    // pub oscillators: Vec<Box<OscBLIT>>,
    // pub voices: Vec<Box<voice::Voice<'a>>>,
    pub oscillators: Vec<OscBLIT>,
    pub voices: Vec<voice::Voice<'a>>,
    polyphony: types::polyphony,
    unison: types::unison,
    nvoices: types::nvoices,
    detune: types::detune,
    pub br: Observable<'a, types::note2osc>,
    pub voicevec: types::note2osc,
    pub maxnotes: Observable<'a, types::maxnotes>,
}

impl<'a> Polyphony<'a> {
    pub fn new() -> Polyphony<'a> {
        let mut p = Polyphony {
            // oscillators: vec![
            //     Box::new(OscBLIT::new()), 
            //     Box::new(OscBLIT::new()), 
            //     Box::new(OscBLIT::new()), 
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            //     Box::new(OscBLIT::new()),
            // ],
            // voices: vec![
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            //     Box::new(voice::Voice::new()),
            // ],
            oscillators: vec![
                OscBLIT::new(), 
                OscBLIT::new(), 
                OscBLIT::new(), 
                OscBLIT::new(),
                OscBLIT::new(),
                OscBLIT::new(),
                OscBLIT::new(),
                OscBLIT::new(),
                OscBLIT::new(),
                OscBLIT::new(),
                OscBLIT::new(),
            ],
            voices: vec![
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
                voice::Voice::new(),
            ],
            polyphony: types::polyphony(false),
            unison: types::unison(false),
            // TODO: must be set to 0 to force an update and subsequent call to 
            // map_oscillators() when the default for nvoices is set 
            // (which is 1, not 0)
            nvoices: types::nvoices(0),
            detune: types::detune(0f32),
            br: Observable::new(types::note2osc([0; NOSC])),
            voicevec: types::note2osc([0; NOSC]),
            maxnotes: Observable::new(types::maxnotes(1)),
        };
        for (i, o) in p.voicevec.0.iter_mut().enumerate() {
            *o = i as u8
        } 
        p
    }

    pub fn printme(&self)  {
        print!("NOTE2OSC: ");
        for i in self.voicevec.0.iter() {
            print!{" {},", i}
        }
        io::stdout().flush().unwrap();
        println!("");
    }

    // monophony,1 voice per note
    pub fn mono(&mut self) {
                self.voicevec.0[0]=0;
    }

    // monophony, multiple voices per note
    pub fn mono_unison(&mut self) {
        for i in 0..NOSC {
            if i <= self.nvoices.0-1 {
                self.voicevec.0[i] = 0;
            } else {
                // don't play these
                self.voicevec.0[i] = 1;
            }
        } 
    }
 
    // polyphony, 1 voice per note
    pub fn poly(&mut self) {
        for i in 0..NOSC {
            self.voicevec.0[i] = i as u8;
            // println!("Polyphony on, unison off, nnotes = {}", self.nnotes);
        }
    }

    // polyphony, 2 voices per note
    pub fn poly_unison(&mut self) {
        println!(" SETTING POLY UNISON ");
        let mut c = 0u8;
        for i in 0..NOSC {
            self.voicevec.0[i] = c;
            c = c + (i as u8) % 2;
        }
    }

    pub fn map_oscillators(&mut self) {
        // polyphony is true
        if self.polyphony.0 {
            if self.unison.0 {
                self.poly_unison();
                if self.nvoices.0 > 1 {
                    self.maxnotes.update(types::maxnotes((self.nvoices.0 / 2) as u8));
                } else {
                    self.maxnotes.update(types::maxnotes(1));
                }
            } else if !self.unison.0 {
                self.poly();
                self.maxnotes.update(types::maxnotes(self.nvoices.0 as u8));
            }
        } else {
        // polyphony is false
            // if unison is true, play nvoices simultaneaously per one note
            if self.unison.0 {
                self.mono_unison();
            } else {
                self.mono();
            }
            self.maxnotes.update(types::maxnotes(1));
        };
        // self.printme();
        self.br.update(self.voicevec);

        // print!("VOICEVEC: ");
        // for v in self.voicevec.0.iter() {
        //     print!(" {}", v)
        // }
        // io::stdout().flush().unwrap();
        // println!(" ");

        let mut c = 0;
        let mut v_old = 0;
        for v in self.voices.iter_mut() {
            for o in v.f0oscs.iter_mut() {
                while o.observers.len() > 0 {
                    o.observers.pop();
                }
            }
        }
        for (i, v) in self.voicevec.0.iter().enumerate() {
            if *v != v_old {
                c = 0;
            }

            // print!("Assigning oscillator {}", i);
            // print!(" to voice {} at f0osc index {}", *v, c);
            // io::stdout().flush().unwrap();
            // println!(" ");

            let o = &mut self.oscillators[i] as *mut OscBLIT;
            unsafe {
                self.voices[(*v) as usize].f0oscs[c].observers.push(&mut *o);
            }
            c = c+1;
            v_old = *v;

        }

    }

}


impl<'a> Observer<types::polyphony> for Polyphony<'a> {
    fn next(&mut self, p: types::polyphony) {
        if self.polyphony.0 != p.0 {
            self.polyphony.0 = p.0;
            self.map_oscillators()
        }
    }
}
        
impl<'a> Observer<types::unison> for Polyphony<'a> {
    fn next(&mut self, u: types::unison) {
        if self.unison.0 != u.0 {
            self.unison.0 = u.0;
            self.map_oscillators()
        }
    }
}

impl<'a> Observer<types::nvoices> for Polyphony<'a> {
    fn next(&mut self, n: types::nvoices) {
        if self.nvoices.0 != n.0 {
            self.nvoices.0 = n.0;
            self.map_oscillators()
        }
    }
}


impl<'a> Observer<types::noteon> for Polyphony<'a> {
    fn next(&mut self, n: types::noteon) {

                let ivoice = n.2 as usize;

                // print!("VOICEVEC: ");
                // for v in self.voicevec.0.iter() {
                //     print!(" {}", v)
                // }
                // io::stdout().flush().unwrap();
                // println!(" ");

                self.voices[ivoice].update_f0(types::f0(n.0));
                self.voices[ivoice].vel = n.1;

                for i in 0..NOSC {
                    let ref fvoice = self.voices[i].f0;
                    let ref fosc = self.oscillators[i].f0;
                    let ref fnoteon = n.0;
                    let ref velvoice = self.voices[i].vel;
                    // print!{"fnoteon: {}, fvoice: {}, velvoice: {}, fosc: {}", fnoteon, fvoice, velvoice, fosc}
                    // io::stdout().flush().unwrap();
                    // println!(" ")
                }

        // // normalize velocity for each voice, depending on the number of
        // // oscillators
        // for i in 0..NOSC {
        //     if self.voicevec.0[i] == n.2 {
        //         self.voices[i].vel = self.voices[i].vel / (nosc as f32)
        //     }
        // }
    }
}

impl<'a> Observer<types::noteoff> for Polyphony<'a> {
    fn next(&mut self, n: types::noteoff) {
        let ivoice = n.1 as usize;
        self.voices[ivoice].vel = 0f32;
    }
}


