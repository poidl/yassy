
use voice;
use voice::*;
use midi;
use midi::*;
use std::collections::VecDeque;

pub struct Synth {
    pub note_queue: VecDeque<[u8;3]>,
    pub voice: voice::Voice,
}

impl Synth {
    pub fn new() -> Synth {
        Synth { 
            note_queue: VecDeque::with_capacity(10),
            voice: voice::Voice::new() 
        }
    }
    pub fn midievent(&mut self, mm: midi::MidiMessage) {

        if mm.noteon() {
            self.note_queue.push_front(*mm);
            self.noteon(mm.f0(), mm.vel())
        } else if mm.noteoff() {
            // check if this note (identified by number/frequency) is queued
            let result = self.note_queue.iter().position(|x| (x as midi::MidiMessage).note_number() == mm.note_number());
            match result {
                Some(i) => {
                    self.note_queue.remove(i);
                    if i == 0 {
                        self.noteoff();
                        if self.note_queue.len() > 0 {
                            let mm = &self.note_queue[0].clone();
                            self.noteon(mm.f0(), mm.vel())
                        }
                    }
                }
                _ => {}
            }
        }
        for mm in &self.note_queue {
            print!(" {}", mm.note_number());
            println!("")
        }
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.voice.set_fs(fs);
    }
    pub fn noteon(&mut self, f0: f32, vel: f32) {
        self.voice.next(f0, vel)
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
