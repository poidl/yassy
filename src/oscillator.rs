use libc;
use utils;

const N: u32 = 2147483648; // 2^31=2147483648;

pub struct PhaseAccumulator {
    // Half width of segment, i.e. 2^(k-1) with k=32.
    n: u32,
    // sample rate
    fs: f64,
    // current index. Note that the index cannot obtain the value n (since it is an i32), but instead ranges between [-2^(k-1),2^(k-1)-1]
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
            fs: 0f64,
            a: 0i32,
            da: 0i32,
            fac_da: 0f64,
        }
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.fs = fs;
        self.fac_da = 2f64 * self.n as f64 / fs;
    }
    pub fn reset(&mut self, f0: f64) {
        // Set index increment of the phase accumulator. (f0/fs) is the fraction of the full segment (length 2N) per sample. Maximum resolvable non-aliased frequency is f0=fs/2, which must correspond to an index increment of N. So da=2*N*f0/fs
        self.da = (f0 * self.fac_da as f64) as i32;
        // Set self.a to -2^(k-1). Could start at 0 instead, but this corresponds to what Frei uses. Note that (N as i32) is negative and equal to i32::min_value(). Is there a one-liner?
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
    fn set_fs(&mut self, f64);
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
    fn set_fs(&mut self, fs: f64) {
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

// ***********************************************************

pub struct OscMulti {
    osc1: OscBasic,
    osc2: OscBLIT,
    pub currentosc: i8,
}

impl OscMulti {
    pub fn new() -> OscMulti {
        let o1 = OscBasic::new();
        let o2 = OscBLIT::new();
        let oc = 2i8;
        OscMulti {
            osc1: o1,
            osc2: o2,
            currentosc: oc,
        }
    }
}

impl Oscillator for OscMulti {
    fn set_fs(&mut self, fs: f64) {
        self.osc1.set_fs(fs);
        self.osc2.set_fs(fs);
    }
    fn reset(&mut self, f0: f32) {
        self.osc1.reset(f0);
        self.osc2.reset(f0 as f64);
    }
    fn get_amp(&mut self) -> f32 {
        let amp = match self.currentosc {
            1i8 => self.osc1.get_amp(),
            2i8 => self.osc2.get_amp(),
            _ => 0f32,
        };
        // println!("newamp: {}",newamp);
        amp
    }
    fn cleanup(&mut self) {
        self.osc2.cleanup();
    }
}


// ****************************************************

const M: usize = 2 * (2700 - 1) + 1;

pub struct OscBLIT {
    // We translate the fundamental frequency f0 from units 1/t to a fraction "fn" of a wavetable with 2N lattice points. fn corresponds to the number of points which are skipped when reading the wavetable,and can therefore be interpreted as a phase increment. The 2N lattice points represent the interval [-pi,pi). The max. resolved freq. is f0=fs/2, i.e. we want a linear function fn with fn(0)=0 and fn(fs/2)=N. It follows that fn(f0)=2N*f0/fs. If a signed integer of k bits is used as phase accumulator, the 2N interval translates to [-2^(k-1),2^(k-1)-1]. I.e. for k=2, the values range from -2 to 1.
    pub m: u32, // number of entries in half-segment of integratied bandlimited impulse
    pub pa: PhaseAccumulator,
    pub b: i32, // amplitude of phase accumulator, phase shifted by N
    pub alpha: u32,
    pub i: i32,
    pub f: *mut f64,
    pub c: f64,
    pub d: f64,
    pub fs: f64, // sample rate
    pub f0: f64, // fundamental frequency
    pub fac_i: f64, // avoid unnecessary runtime multiplication
    pub fac_alpha: f64,
    pub fac_fn: f64,
    pub abs_a: i32,
}

impl OscBLIT {
    pub fn new() -> OscBLIT {
        unsafe {
            OscBLIT {
                m: M as u32,
                pa: PhaseAccumulator::new(),
                fs: 0f64, // sample rate
                c: 0f64,
                fac_i: 0f64, // avoid unnecessary runtime multiplication
                fac_alpha: 0f64,
                fac_fn: 0f64,
                b: 0i32, // a, phase shifted by N (start at 0)
                f0: 0f64, // fundamental frequency
                alpha: 0u32,
                i: 0i32,
                f: (*Box::into_raw(vec![0f64; M].into_boxed_slice())).as_mut_ptr(), /* TODO: simpler? */
                d: 0f64,
                abs_a: 0i32,
            }
        }
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.pa.set_fs(fs);
        self.fs = fs;
        println!("************* fs: {}", fs);
        let c = 4 as f64 * self.pa.n as f64;
        self.fac_i = self.m as f64 * fs / c; // m*fs/c= m*fs/(4*n)
        self.fac_alpha = c / fs;
        self.fac_fn = 2f64 * self.pa.n as f64 / self.fs;

        let halfsegment = utils::blit_2t(fs);
        unsafe {
            for ii in 0..M {
                *self.f.offset(ii as isize) = halfsegment[ii];
            }
        }
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
        self.abs_a = a ^ mask; // xor with mask is equivalent to -1*(a+1) for a<0, and a no-op otherwise. http://stackoverflow.com/questions/12041632/how-to-compute-the-integer-absolute-value
    }

    // Compute alpha
    pub fn set_alpha_i(&mut self) {
        // TODO: alpha is just twice the phase increment fnn
        self.alpha = (self.f0 * self.fac_alpha) as u32; // self.alpha = 0..2N

    }

    // Compute and store segment value if |A| < alpha.
    pub fn step_c(&mut self) {

        if self.abs_a < (self.alpha as i32) {

            // Find the appropriate index i of the half segment. Imagining a horizontal "index" axis, the full segment is antisymmetric around index 0. We only stored the right half. Since |A| < alpha, |A|/alpha = 0..1. We map A/alpha linearly onto [0,M-1].
            let i = (self.m as f64 - 1f64) * self.abs_a as f64 / (self.alpha as f64);
            // Convert to integer.
            let i = i.trunc() as i32;

            self.i = i;
            unsafe {
                self.c = *self.f.offset(self.i as isize);
            }
        } else {
            self.c = 0f64;
        }
    }
    pub fn step_d(&mut self) {
        let n = self.pa.n as f64;
        if self.b > 0i32 {
            self.d = self.c + self.b as f64 / n;
        } else {
            self.d = -self.c + self.b as f64 / n;
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
    fn set_fs(&mut self, fs: f64) {
        self.set_fs(fs);
    }
    fn reset(&mut self, f0: f32) {
        self.reset(f0 as f64);
    }
    fn get_amp(&mut self) -> f32 {
        self.get() as f32
    }
    fn cleanup(&mut self) {
        unsafe { libc::free(self.f as *mut libc::c_void) }
    }
}
