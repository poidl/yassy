use utils;
use types;

use midi;
use midi::*;
use observer;
use observer::*;
use std::ptr;

const N: u32 = 2147483648; // 2^31=2147483648;

pub struct PhaseAccumulator {
    // Half width of segment, i.e. 2^(k-1) with k=32.
    n: u32,
    // sample rate
    fs: types::fs,
    // current index. Note that the index cannot obtain the value n (since
    // it is an i32), but instead ranges between [-2^(k-1),2^(k-1)-1]
    pub a: i32,
    // index increment
    da: i32,
    // avoid unnecessary runtime multiplication
    fac_da: f64,
}

impl PhaseAccumulator {
    pub fn new() -> PhaseAccumulator {
        PhaseAccumulator {
            n: 2147483648,
            fs: types::fs(0f64),
            a: 0i32,
            da: 0i32,
            fac_da: 0f64,
        }
    }
    pub fn set_fs(&mut self, fs: types::fs) {
        self.fs = fs;
        self.fac_da = 2f64 * self.n as f64 / fs.0;
    }
    pub fn reset(&mut self, f0: f64) {
        // Set index increment of the phase accumulator. (f0/fs) is the
        // fraction of the full segment (length 2N) per sample. Maximum
        // resolvable non-aliased frequency is f0=fs/2, which must
        // correspond to an index increment of N. So da=2*N*f0/fs
        self.da = (f0 * self.fac_da as f64) as i32;
        // Set self.a to -2^(k-1). Could start at 0 instead, but this
        // corresponds to what Frei uses. Note that (N as i32) is negative
        // and equal to i32::min_value().
        self.a = 0i32;
        self.a = self.a.wrapping_add(N as i32);
    }
    pub fn get_a(&mut self) -> i32 {
        self.step();
        self.a
    }
    pub fn step(&mut self) {
        self.a = self.a.wrapping_add(self.da);
    }
    pub fn shiftn(&mut self) -> i32 {
        self.a.wrapping_add(self.n as i32)
    }
    pub fn normalize_index(&mut self) -> f64 {
        self.a as f64 / self.n as f64
    }
}

// **********************************************************

pub trait Oscillator {
    fn set_fs(&mut self, types::fs);
    fn reset(&mut self, f32);
    fn get_amp(&mut self) -> f32;
    fn cleanup(&mut self);
}

// **********************************************************

pub struct OscBasic {
    pa: PhaseAccumulator,
}

impl OscBasic {
    pub fn new() -> OscBasic {
        OscBasic { pa: PhaseAccumulator::new() }
    }
}

/// Make an Oscillator from a PhaseAccumulator by adding get_amp()
/// and cleanup(). Non-bandlimited, for testing only.

impl Oscillator for OscBasic {
    fn set_fs(&mut self, fs: types::fs) {
        self.pa.set_fs(fs)
    }
    fn reset(&mut self, f0: f32) {
        self.pa.reset(f0 as f64)
    }
    fn get_amp(&mut self) -> f32 {
        self.pa.step();
        let phi: f32 = (self.pa.a as f64 / self.pa.n as f64) as f32;
        return phi;
    }
    fn cleanup(&mut self) {}
}

// ****************************************************

const M: usize = 2 * (2700 - 1) + 1;

pub struct OscBLIT {
    // We translate the fundamental frequency f0 from units 1/t to a
    // fraction "fn" of a wavetable with 2N lattice points. fn corresponds
    // to the number of points which are skipped when reading the wavetable,
    // and can therefore be interpreted as a phase increment. The 2N
    // lattice points represent the interval [-pi,pi). The max. resolved
    // freq. is f0=fs/2, i.e. we want a linear function fn with fn(0)=0 and
    // fn(fs/2)=N. It follows that fn(f0)=2N*f0/fs. If a signed integer of
    // k bits is used as phase accumulator, the 2N interval translates to
    // [-2^(k-1),2^(k-1)-1]. I.e. for k=2, the values range from -2 to 1.

    // number of entries in half-segment of integratied bandlimited impulse
    pub m: u32,
    pub pa: PhaseAccumulator,
    // amplitude of phase accumulator, phase shifted by N
    pub b: i32,
    pub alpha: u32,
    pub i: i32,
    pub c: f64,
    pub d: f64,
    // sample rate
    pub fs: types::fs,
    // fundamental frequency
    pub f0: f64,
    // avoid unnecessary runtime multiplication
    pub fac_i: f64,
    pub fac_alpha: f64,
    pub fac_fn: f64,
    pub abs_a: i32,
    pub blit_segment: [f64; M],
    pub use_blit: bool,
    pub use_postfilter: bool,
    pub pf_b0: f64,
    pub pf_a1: f64,
    pub d_old: f64,
    // TODO: Producers allocate boxes?
    pub buf: Box<f32>
}

// Postfilter (Frei p. 17)
// H( z ) = 1/(0.65 + 0.35*z^(-1)) = 1.538462/(1+0.538462*z^(-1)) :=
// := b0/(1+a1*z^(-1))
// b0=1.538462, a1=0.538462
// y(n) = b0*x(n)-a1*y(n-1) = b0*x(n)-a1*d_old


impl OscBLIT {
    pub fn new() -> OscBLIT {
        OscBLIT {
            m: M as u32,
            pa: PhaseAccumulator::new(),
            // sample rate
            fs: types::fs(0f64),
            c: 0f64,
            // avoid unnecessary runtime multiplication
            fac_i: 0f64,
            fac_alpha: 0f64,
            fac_fn: 0f64,
            // a, phase shifted by N (start at 0)
            b: 0i32,
            // fundamental frequency
            f0: 0f64,
            alpha: 0u32,
            i: 0i32,
            d: 0f64,
            abs_a: 0i32,
            blit_segment: [0f64; M],
            use_blit: true,
            use_postfilter: true,
            pf_b0: 1.538462f64,
            pf_a1: 0.538462f64,
            d_old: 0f64,
            // buf: &mut 0f32 as *mut f32
            buf: Box::new(0f32)
        }
    }
    pub fn set_fs(&mut self, fs: types::fs) {
        self.pa.set_fs(fs);
        self.fs = fs;
        println!("************* fs: {}", fs.0);
        let c = 4 as f64 * self.pa.n as f64;
        // m*fs/c= m*fs/(4*n)
        self.fac_i = self.m as f64 * fs.0 / c;
        self.fac_alpha = c / fs.0;
        self.fac_fn = 2f64 * self.pa.n as f64 / self.fs.0;

        self.blit_segment = utils::blit_2t(fs.0);

    }

    pub fn reset(&mut self, f0: f64) {
        self.pa.reset(f0);
        self.b = self.pa.shiftn();
        self.f0 = f0;
    }
    pub fn step_ab(&mut self) {
        self.pa.step();
        self.b = self.pa.shiftn();
        // a.abs() will panic/overflow if a=i32::min_value().
        let a = self.pa.a;
        let mask = a >> 31u32;
        // xor with mask is equivalent to -1*(a+1) for a<0, and a no-op
        // otherwise. http://stackoverflow.com/questions/12041632
        self.abs_a = a ^ mask;
    }

    // Compute alpha
    pub fn set_alpha_i(&mut self) {
        // TODO: alpha is just twice the phase increment fnn
        // self.alpha = 0..2N
        self.alpha = (self.f0 * self.fac_alpha) as u32;

    }

    // Compute and store segment value if |A| < alpha.
    pub fn step_c(&mut self) {

        if self.abs_a < (self.alpha as i32) {

            // Find the appropriate index i of the half segment. Imagining
            //  a horizontal "index" axis, the full segment is
            // antisymmetric around index 0. We only stored the right
            // half. Since |A| < alpha, |A|/alpha = 0..1. We map A/alpha
            // linearly onto [0,M-1].
            let i = (self.m as f64 - 1f64) * self.abs_a as f64 / (self.alpha as f64);
            // Convert to integer.
            let i = i.trunc() as i32;

            self.i = i;
            self.c = self.blit_segment[self.i as usize];
        } else {
            self.c = 0f64;
        }
    }
    pub fn step_d(&mut self) {
        let n = self.pa.n as f64;
        self.d = self.b as f64 / n;
        if self.use_blit {
            if self.b > 0i32 {
                self.d += self.c;
            } else {
                self.d -= self.c;
            }
        }
        // y(n) = b0 * x(n) - a1 * d_old
        if self.use_postfilter {
            self.d = self.pf_b0 * self.d - self.pf_a1 * self.d_old;
            self.d_old = self.d;
        }

    }

    pub fn get(&mut self) -> f64 {
        self.step_ab();
        self.set_alpha_i();
        self.step_c();
        self.step_d();
        self.d as f64
    }
}

impl Oscillator for OscBLIT {
    fn set_fs(&mut self, fs: types::fs) {
        self.set_fs(fs);
    }
    fn reset(&mut self, f0: f32) {
        self.reset(f0 as f64);
    }
    fn get_amp(&mut self) -> f32 {
        self.get() as f32
    }
    fn cleanup(&mut self) {}
}

// ***********************************************************

// pub struct OscMulti {
//     osc1: OscBasic,
//     osc2: OscBLIT,
//     pub currentosc: i8,
// }

// impl OscMulti {
//     pub fn new() -> OscMulti {
//         let o1 = OscBasic::new();
//         let o2 = OscBLIT::new();
//         let oc = 2i8;
//         OscMulti {
//             osc1: o1,
//             osc2: o2,
//             currentosc: oc,
//         }
//     }
// }

// impl Oscillator for OscMulti {
//     fn set_fs(&mut self, fs: types::fs) {
//         self.osc1.set_fs(fs);
//         self.osc2.set_fs(fs);
//     }
//     fn reset(&mut self, f0: f32) {
//         self.osc1.reset(f0);
//         self.osc2.reset(f0 as f64);
//     }
//     fn get_amp(&mut self) -> f32 {
//         let amp = match self.currentosc {
//             1i8 => {
//                 self.osc2.use_postfilter = false;
//                 self.osc2.get_amp()
//             }
//             2i8 => {
//                 self.osc2.use_postfilter = true;
//                 self.osc2.get_amp()
//             }
//             _ => 0f32,
//         };
//         // println!("newamp: {}",newamp);
//         amp
//     }
//     fn cleanup(&mut self) {
//         self.osc2.cleanup();
//     }
// }

impl Observer<MidiMessage> for OscBLIT {
    fn next(&mut self, mm: midi::MidiMessage) {
 println!(" SETTING NOTE: {}", mm.f0());
        if mm.noteon() {
            self.reset(mm.f0() as f64);
        }
    }
}
impl Observer<types::f0> for OscBLIT {
    fn next(&mut self, f0: types::f0) {
 println!(" OSC RECEIVED FREQUENCY: {}", f0.0 as f32);
        self.reset(f0.0 as f64);
    }
}

impl Observer<types::fs> for OscBLIT {
    fn next(&mut self, fs: types::fs) {
        self.set_fs(fs);
    }
}
impl Observer<u32> for OscBLIT {
    fn next(&mut self, _pos: u32) {
        unsafe {
            let amp = self.get_amp();
            // println!("******** AMP: {}", amp);
            // self.buf = &mut self.get_amp() as *mut f32;
            *self.buf = amp;
        }
    }
}
