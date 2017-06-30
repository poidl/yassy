#[derive(Copy, Clone)]
pub struct fs(pub f64);
pub struct gain(f32);
pub struct note((f32,f32));
#[derive(Copy, Clone)]
pub struct noteon(pub f32, pub f32);
#[derive(Copy, Clone)]
pub struct noteoff(pub u8);
#[derive(Copy, Clone)]
pub struct f0(pub f32);
// pub struct gain(f32);
