extern crate yassy;

const N: u32 = 2147483648;

#[test]
fn mytest() {

    // Check if type conversion overflows behave as expected
    assert_eq!(N as i32, i32::min_value());
    assert_eq!((N as i32) as i64, -(N as i64));

    let mut pa = yassy::oscillator::PhaseAccumulator::new();

    // set f0 to exactly half the sample rate and verify phase accumulation. Exactly two steps for one segment cycle.
    pa.set_fs(44100f64);
    pa.reset(22500f64);
    // Check if index is initialized to -N
    assert_eq!(pa.a as i64, -(N as i64));
    // step once ... index must be zero
    let mut a = pa.get_a();
    assert_eq!(a, 0i32);
    // step once ... must be back at the left end of the segment
    a = pa.get_a();
    assert_eq!(a as i64, -(N as i64));
    // check if shiftn works
    assert_eq!(pa.shiftn(), 0i32);
}
