extern crate libc;

pub fn rgsl_sf_bessel_io(x: f64) -> f64 {
    unsafe { gsl_sf_bessel_I0(x) }
}

#[link(name = "gsl")]
#[link(name = "gslcblas")]
extern "C" {
    /// This routine computes the regular modified cylindrical Bessel function of zeroth order, I_0(x)
    pub fn gsl_sf_bessel_I0(x: libc::c_double) -> libc::c_double;
}