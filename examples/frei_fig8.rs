#![allow(unused_variables)]
extern crate yassy;
extern crate mygsl;
extern crate plot;

use yassy::utils;
use yassy::utils::*;
use std::f64;
use plot::*;

fn main() {

    // This approximately reproduces figure 8 in "Digital sound generation":
    // Frei, B.: Digital sound generation. Institute for Computer Music and Sound Technology (ICST) Zurich University of the Arts.

    // Each sampling interval T is represented by exactly nppt points (number of
    // points per T). Denoting the resolution of the temporal lattice as Tl and
    // the corresponding frequency as fl, it follows that T=(nppt-1)*Tl, and
    // fs=1/T=fl/(nppt-1), where fl=1/Tl.
    // nt is number of sampling intervals T on the temporal lattice, whose length
    // is N=nt*nppt-1. Note that a point representing the end of one T represents
    // also the starting point of the next T.
    // The time signal is symmetric around 0, and if we want 0 to coincide
    // exactly with a lattice point, then nt must be an even number. It follows
    // from N=nt*(nppt-1)+1 that N must be uneven. This is somewhat inconvenient,
    // because the most efficient fft algorithms only accept vectors with a
    // length that is a power of 2 (and hence is even). We therefore append
    // some points to the time series until its length becomes a power of 2. This
    // will also
    // increase the frequency resolution. See the book of Frei, especially
    // the appendix with the MATLAB code listing. Note that we use a different
    // number of points, because we have to use the real_radix2 fft algorithm of gsl.

    let nt: usize = 10;
    let nppt: usize = 100;
    // N needs to be a constant if we want to stay in the heap memory (2016/01/22)
    // Can't use variables nt and nppt
    const N: usize = 10*(100-1)+1; // Formula: nt*(nppt-1)+1 as usize;
    if N != nt*(nppt-1)+1 {
        panic!("inconsistent variables");
    }
    const NN: usize =1048576; // 2u32.pow(20u32)=1048576
    let fs = 48000f64; // sampling frequency 1/T.
    let fc = 18300f64; // cutoff frequency 1/Tc.

    let pi = std::f64::consts::PI;
    let alpha = 9f64/pi; // alpha for Kaiser window. Note that beta = pi*alpha.
    let alpha_apo = 0.7f64/pi; // apodization
    let apof = 0.9f64;

    let nipt = (nppt-1) as f64; // number of Tl per T

    // as time axis we use units of Tl instead of T for the calculation, to keep
    // the formula for the Fourier trafo clean (linear axis scaling). Later
    // in the plot we will drop the factor nipt and display in units of T.
    let nth = 0.5f64*(nt as f64)*nipt;

    //let t = utils::linspace_heapslice(-nth , nth , N);
    let mut t : [f64;N] = [0f64;N];
    utils::linspace(&mut t,-nth, nth);

    // The first zero crossing of the impulse determines the cutoff frequency fc=(1/Tc).
    // The normalized sinc function sinc(T) has the first zero crossing at 1*T,
    // corresponding to a cutoff at 0.5(1/T)=0.5*fs, which differs by a factor of
    // c=2*fc/fs from fc.  To get the cutoff at fc, we use c*sinc(c*T) (see doc for
    // the effect of linear axis scaling on the Fourier transform). Also include
    // a factor 1/nipt to transform T into Tl

    let c =  2f64*fc/(fs*nipt);

    // impulse h
    t.mult(&c); let mut ct=t;
    ct.sinc(); let mut sinc_ct= ct;
    sinc_ct.mult(&c); let mut h = sinc_ct;

    // apply Kaiser window
    h.kaiser(alpha); let mut hk = h;

    // apodization (scale the maximum to c afterwards)
    let mut kaiser_apo : [f64;N] = [1f64;N];
    kaiser_apo.kaiser(alpha_apo);

    let mut apo : [f64;N] = [f64::NAN;N];
    for i in 0..apo.len() {
        apo[i]=1f64-apof*kaiser_apo[i];
    }
    hk.mult(&apo[..]);

    let mut tmp : [f64;N] = [0f64;N];
    tmp.mycopy(&hk);
    tmp.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let max=tmp[N-1];
    hk.mult(&(c*(1f64/max)));

    // Before Fourier transforming h to fh, append points to make the length a
    // power of 2

    let mut fh = [0f64;NN];
    for i in 0..hk.len() {
        fh[i]=hk[i];
    }
    mygsl::fft::real_radix2::transform(&mut fh,1,NN);
    //
    // magnitude (abs) of Re and Im
    let mut fhabs : [f64;NN/2 +1]=[f64::NAN;NN/2 +1];
    for ii in 0..NN/2 {
        fhabs[ii]=(fh[ii].powf(2f64)+fh[NN-1-ii].powf(2f64)).sqrt();
    }

    // print out some usuful numbers
    let npptf64 = nppt as f64;
    println!("N:    {}", N);
    println!("fs:   {}", fs);
    println!("nppt: {}", nppt);
    println!("fl:   {}", fs*(npptf64-1f64));
    println!("fl/2: {}", fs*(npptf64-1f64)/2f64);
    let outname = "./examples/figures/frei_fig8.svg";
    plot::plot_ampl_spec(nt, nppt, NN, fs, &fhabs, outname)
}
