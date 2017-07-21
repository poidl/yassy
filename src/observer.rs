use midi;
use midi::*;
use std::iter::*;
use std::rc::Rc;
use std::marker::Copy;

pub trait NoteEventObserver {
    fn next(&mut self, f0: f32, vel: f32);
}

pub type Observers<'a, T> = Vec<&'a mut Observer<T>>;

pub trait Observer<T> {
    fn next(&mut self, mm: T);
    // fn next_noteon(&mut self, f0: f32, vel: f32);
    // fn next_noteoff(&mut self);
    // fn next_fs(&mut self, fs: f64);
    // fn next_blit(&mut self, blit: bool);
    // fn next_postfilter(&mut self, pf: bool);
    // fn get_amp(&mut self) -> f32;
    // fn cleanup(&mut self);
}

// pub trait ObserverTmp  {
//     fn next_noteoff(&mut self);
//     fn next_fs(&mut self, fs: f64);
//     fn next_blit(&mut self, blit: bool);
//     fn next_postfilter(&mut self, pf: bool);
//     fn get_amp(&mut self) -> f32;
//     fn cleanup(&mut self);
// }

pub struct Observable<'a, T: 'a> {
    item: T,
    pub observers: Observers<'a, T>,
}

impl<'a, T: Copy> Observable<'a, T> {
    pub fn new(item: T) -> Observable<'a, T> {
        Observable {
            item: item,
            observers: Vec::with_capacity(10),
        }
    }
    pub fn update(&mut self, item: T) {
        self.item = item;
        self.notify();
    }
    pub fn notify(&mut self) {
        // println!("notifying:::::::::::::::::::::::::;");
        for o in self.observers.iter_mut() {
            // println!(" notifying::::::::");
            o.next(self.item);
            // println!("::::::::");
        }

    }
}

// impl Observers {
//     pub fn push(&mut self, obs: Box<Observer>) {
//         self.0.push(obs);
//     }
// }


pub trait ObservableTrait<T> {
    fn notify(&mut self, item: T) {}
    fn notifyevent_noteon(&mut self, f0: f32, vel: f32) {}
    fn notifyevent_noteoff(&mut self) {}
    fn notifyevent_fs(&mut self, fs: f64) {}
    fn notifyevent_blit(&mut self, blit: bool) {}
    fn notifyevent_postfilter(&mut self, pf: bool) {}
}


// impl<T> ObservableTrait<T> for Observable<T> where T:'a {
//     fn notify(&mut self, item: T) {
//         for o in &mut self.observers {
//             o.next(item)
//         }
//     }
//     fn notifyevent_noteon(&mut self, f0: f32, vel: f32) {
//         for o in &mut self.observers {
//             o.next_noteon(f0, vel)
//         }
//     }
//     fn notifyevent_noteoff(&mut self) {
//         for o in &mut self.observers {
//             o.next_noteoff()
//         }
//     }
//     fn notifyevent_fs(&mut self, fs: f64) {
//         for o in &mut self.observers {
//             o.next_fs(fs)
//         }
//     }
//     fn notifyevent_blit(&mut self, blit: bool) {
//         for o in &mut self.observers {
//             o.next_blit(blit)
//         }
//     }
//     fn notifyevent_postfilter(&mut self, pf: bool) {
//         for o in &mut self.observers {
//             o.next_postfilter(pf)
//         }
//     }
// }

// fn next(&mut self, item: T) {
//     self.on = true;
//     self.f0 = f0;
//     self.vel = vel;
//     self.initialize();
// }
