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

const NOSC: usize = 4;

pub struct Polyphony<'a> {
    pub oscillators: Vec<Box<OscBLIT>>,
    pub voices: Vec<Box<voice::Voice<'a>>>,
    polyphony: types::polyphony,
    unison: types::unison,
    nvoices: types::nvoices,
    detune: types::detune,
    pub br: Observable<'a, types::note2osc>,
    voicevec: types::note2osc,
    pub maxnotes: Observable<'a, types::maxnotes>,
}

impl<'a> Polyphony<'a> {
    pub fn new() -> Polyphony<'a> {
        let mut p = Polyphony {
            oscillators: vec![
                Box::new(OscBLIT::new()), 
                Box::new(OscBLIT::new()), 
                Box::new(OscBLIT::new()), 
                Box::new(OscBLIT::new())
            ],
            voices: vec![
                Box::new(voice::Voice::new()),
                Box::new(voice::Voice::new()),
                Box::new(voice::Voice::new()),
                Box::new(voice::Voice::new()),
            ],
            polyphony: types::polyphony(false),
            unison: types::unison(false),
            nvoices: types::nvoices(1),
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
            self.voicevec.0[i] = 0;
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
        // println!(" SETTING POLY UNISON ");
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

        // while self.noteon.observers.len > 0 {
        //     self.noteon.observers.pop()
        // }

        println!("map ********************");
        for (i, v) in self.voicevec.0.iter().enumerate() {
                let o = &mut*self.oscillators[i] as *mut OscBLIT;
                unsafe {
                    self.voices[(*v) as usize].f0.observers.push(&mut *o);
                }
        }

    }

}


impl<'a> Observer<types::polyphony> for Polyphony<'a> {
    fn next(&mut self, p: types::polyphony) {
        self.polyphony.0 = p.0;
        self.map_oscillators()
    }
}
        

impl<'a> Observer<types::nvoices> for Polyphony<'a> {
    fn next(&mut self, n: types::nvoices) {
        self.nvoices = types::nvoices(n.0);
        // println!("NOSC = {}", self.NOSC);
    }
}

impl<'a> Observer<types::unison> for Polyphony<'a> {
    fn next(&mut self, u: types::unison) {
        self.unison.0 = u.0;
        self.map_oscillators()
    }
}

impl<'a> Observer<types::noteon> for Polyphony<'a> {
    fn next(&mut self, n: types::noteon) {

        let mut nosc = 0u8;
        for i in 0..NOSC {
            if self.voicevec.0[i] == n.2 {
                println!{"Set f0 for oscillator {}", i}
                self.voices[i].f0.update(types::f0(n.0));
                self.voices[i].vel = n.1;
                nosc = nosc + 1;
            }
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

        let mut nosc = 0u8;
        for i in 0..NOSC {
            if self.voicevec.0[i] == n.1 {
                self.voices[i].vel = 0f32;
            }
        }
    }
}


