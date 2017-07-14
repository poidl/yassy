use types;
use std::collections::VecDeque;
use observer;
use observer::*;
use midi;
use midi::*;
use std::io;
use std::io::Write;

const NOSC: usize = 4;

pub struct Polyphony<'a> {
    pub voices: Vec<Box<voice::Voice<'a>>>,
    polyphony: types::polyphony,
    unison: types::unison,
    nvoices: types::nvoices,
    detune: types::detune,
    pub br: Observable<'a, types::note2osc>,
    voicevec: types::note2osc
    pub maxnotes: Observable<'a, types::maxnotes>,
}

impl<'a> Polyphony<'a> {
    pub fn new() -> Polyphony<'a> {
        let mut p = Polyphony {
            voices: vec![Box::new(voice::Voice::new()); NOSC],
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
        self.unison.0 = u.0;
        self.map_oscillators()
    }
}


