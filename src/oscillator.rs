
use utils;

pub trait Oscillator {
    fn set_fs(&mut self, f64);
    fn reset(&mut self, f32);
    fn get_amp(&mut self) -> f32;
}

impl Oscillator for OscBasic {
    fn set_fs(&mut self, fs: f64) {
        self.fs = fs;
    }
    fn reset(&mut self, f0: f32) {
		// Phase increment of the phase accumulator. (f0/fs) is the
        // fraction of period per sample. This is multiplied by 2^32, so
        // each frequency is equivalent to a fraction of the "maximum
        // phase increment" 2^32, which corresponds to  f0 = fs.
		// (2^32)/16=268435456
        self.dphase =  ((f0/self.fs as f32)*4294967296.0) as u32;
        self.phase =  0
    }
    fn get_amp(&mut self) -> f32 {
        self.step();
        let phi: f32 = (self.phase as f64/2147483648.0 -1f64) as f32;
        return phi
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
            _ => 0f32
        };        
        // println!("newamp: {}",newamp);
        amp
    }
}

pub struct OscMulti {
    osc1: OscBasic,
    osc2: OscBLIT,
    pub currentosc: i8
}

impl OscMulti {
    pub fn new() -> OscMulti {
        let o1 = OscBasic::new();
        let o2 = OscBLIT::new();
        let oc= 2i8;
        OscMulti {osc1: o1, osc2: o2, currentosc: oc}
    }
}

pub struct OscBasic {
    fs: f64,
    pub phase: u32,
    pub dphase: u32
}

impl OscBasic {
    fn step(&mut self){
        self.phase = self.phase.wrapping_add(self.dphase);
    }
}

impl OscBasic {
    pub fn new() -> OscBasic {
        OscBasic {
            fs: 0f64,
            phase: 0u32,
            dphase: 0u32
        }
    }
}

pub struct OscBLIT {
    // We translate the fundamental frequency f0 from units 1/t to a fraction "fn" of a wavetable with 2N lattice points. fn corresponds to the number of points which are skipped when reading the wavetable,and can therefore be interpreted as a phase increment. The 2N lattice points represent the interval [-pi,pi). The max. resolved freq. is f0=fs/2, i.e. we want a linear function fn with fn(0)=0 and fn(fs/2)=N. It follows that fn(f0)=2N*f0/fs. If a signed integer of k bits is used as phase accumulator, the 2N interval translates to [-2^(k-1),2^(k-1)). Note that the interval is open on the left. For k=2, the values range from -2 to 1.
    pub n: u32,
    pub a: i32, // phase. Wavetable size is 2N. start at zero, wrap at N from 1 to -1
    pub fnn: u32, // phase increment
    pub b: i32, // a, phase shifted by N
    pub alpha: u32,
    pub m: u32, // number of entries in half-segment of integratied bandlimited impulse
    pub i: i32,
    pub f: *const f64,
    pub c: f64,
    pub d: f64,
    pub fs: f64, // sample rate
    pub f0: f64, // fundamental frequency
    pub fac_i: f64, // avoid unnecessary runtime multiplication
    pub fac_alpha: f64,
    pub fac_fn: f64,
    pub abs_a: i32
}

impl OscBLIT {
    pub fn new() -> OscBLIT {
        unsafe{
            OscBLIT {
                n: 2u32.pow(31), // follow notation of Frei (p. 3)
                m: (2*(2700-1)+1) as u32,
                fs: 0f64, // sample rate            
                c: 0f64,    
                fac_i: 0f64, // avoid unnecessary runtime multiplication
                fac_alpha: 0f64,
                fac_fn: 0f64,
                        
                a: 0i32, // phase. Wavetable size is 2N. start at -N, wrap at N from 1 to -1
                b: 0i32, // a, phase shifted by N (start at 0)
                f0: 0f64, // fundamental frequency
                fnn: 0u32, // phase increment
                
                alpha: 0u32,
                i: 0i32,
                f: (*Box::into_raw(utils::blit_4t())).as_ptr(), // TODO: this must be cleaned up? See https://doc.rust-lang.org/std/primitive.pointer.html
                d: 0f64,
                abs_a: 0i32
            }            
        }
    }
    pub fn set_fs(& mut self, fs: f64) {
        self.fs = fs;
        println!("************* fs: {}", fs);
        let c = 4 as f64 * self.n as f64;
        self.fac_i = self.m as f64 *fs/c; // m*fs/c= m*fs/(4*n)
        self.fac_alpha = c/fs;
        self.fac_fn = 2f64*self.n as f64/self.fs;
    }
    pub fn reset(&mut self, f0: f64) {
        self.b =  0;
        self.a =  self.b.wrapping_add(self.n as i32);
        self.f0=f0;
        self.fnn =  (f0*self.fac_fn) as u32; // 2*N*f0/fs
    }
    // pub fn set_f0fn(&mut self, f0: f64) {
    //     self.f0 = f0;
    //     self.fnn =  (f0*self.fac_fn) as u32;
    // }
    pub fn step_ab(&mut self){
        // wrapping_add: allows intentional overflow
        self.b = self.b.wrapping_add(self.fnn as i32);
        self.a = self.b.wrapping_add(self.n as i32);
        // a.abs() will panic/overflow if a=i32::min_value().
        let mask = self.a >> 31u32;
        self.abs_a = self.a ^ mask; // xor with mask is equivalent to -1*(a+1) for a<0, and a no-op otherwise. http://stackoverflow.com/questions/12041632/how-to-compute-the-integer-absolute-value
    }
    pub fn set_alpha_i(&mut self) {
        // TODO: alpha is just twice the phase increment fnn
        self.alpha =  (self.f0*self.fac_alpha) as u32; // self.alpha = 0..2N
        let tmp = self.m as f64* (1f64 + self.a as f64 /(self.alpha as f64));
        if self.abs_a < (self.alpha as i32) {
            println!("APPLY A/alpha: {}", self.a as f64/self.alpha as f64); 
            println!("segment index (0..2): {}", (tmp as f64)/(self.m as f64));
        }
        self.i = tmp.trunc() as i32;
    }
    pub fn step_c(&mut self) {
        if self.abs_a < (self.alpha as i32) {
            unsafe {
                // TODO: valgrind shows invalid reads in the following line:
                self.c = -*self.f.offset(self.i as isize);
            }
            // println!("apply {}", self.c);
        } else {
            self.c = 0f64;
        }
    }
    pub fn step_d(&mut self) {
        let n = self.n as f64;
        // println!("self.b {}", self.b as f64/ n );
        // println!("self.c {}", self.c);
        // println!("self.i {}", self.i);
        // println!(" ");
        // self.c = 0f64;
        self.d = self.c + self.b as f64/ n
    }
    pub fn get(&mut self) -> f64 {
        self.step_ab();
        self.set_alpha_i();
        self.step_c();
        self.step_d();
        self.d as f64
    }
}
