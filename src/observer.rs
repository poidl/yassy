use midi;
use midi::*;
use std::iter::*;

pub trait NoteEventObserver {
    fn next(&mut self, f0: f32, vel: f32);
}

pub type Observers = Vec<Box<Observer>>;

pub struct Observable<T> {
    item: T,
    observers: Observers,
}

// impl Observers {
//     pub fn push(&mut self, obs: Box<Observer>) {
//         self.0.push(obs);
//     }
// }

pub trait Observer:  {
    fn next(&mut self, mm: MidiMessage);
    fn next_noteon(&mut self, f0: f32, vel: f32);
    fn next_noteoff(&mut self);
    fn next_fs(&mut self, fs: f64);
    fn next_blit(&mut self, blit: bool);
    fn next_postfilter(&mut self, pf: bool);
    fn get_amp(&mut self) -> f32;
    fn cleanup(&mut self);
}

pub trait ObservableTrait {
    fn notifyevent_noteon(&mut self, f0: f32, vel: f32) {}
    fn notifyevent_noteoff(&mut self) {}
    fn notifyevent_fs(&mut self, fs: f64) {}
    fn notifyevent_blit(&mut self, blit: bool) {}
    fn notifyevent_postfilter(&mut self, pf: bool) {}
}

impl<T> ObservableTrait for Observable<T> {
    fn notifyevent_noteon(&mut self, f0: f32, vel: f32) {
        for o in &mut self.observers {
            o.next_noteon(f0, vel)
        }
    }
    fn notifyevent_noteoff(&mut self) {
        for o in &mut self.observers {
            o.next_noteoff()
        }
    }
    fn notifyevent_fs(&mut self, fs: f64) {
        for o in &mut self.observers {
            o.next_fs(fs)
        }
    }
    fn notifyevent_blit(&mut self, blit: bool) {
        for o in &mut self.observers {
            o.next_blit(blit)
        }
    }
    fn notifyevent_postfilter(&mut self, pf: bool) {
        for o in &mut self.observers {
            o.next_postfilter(pf)
        }
    }
}

// fn next(&mut self, item: T) {
//     self.on = true;
//     self.f0 = f0;
//     self.vel = vel;
//     self.initialize();
// }
