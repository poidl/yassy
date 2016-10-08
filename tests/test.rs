extern crate  yassy;
use yassy::*;

const N: u32 = 2147483648;

#[test]
fn mytest() {
    let mut pa = yassy::oscillator::PhaseAccumulator::new(N);
	pa.set_fs(44100f64);
	pa.reset(22500f64);
	assert_eq!(pa.i,-(N as i32));
}
