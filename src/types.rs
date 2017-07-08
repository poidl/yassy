#[derive(Copy, Clone)]
pub struct fs(pub f64);

#[derive(Copy, Clone)]
pub struct parameter(f32);
#[derive(Copy, Clone)]
pub struct blit(pub bool);
#[derive(Copy, Clone)]
pub struct gain(pub f32);
#[derive(Copy, Clone)]
pub struct polyphony(pub bool);
#[derive(Copy, Clone)]
pub struct unison(pub bool);
#[derive(Copy, Clone)]
pub struct detune(pub f32);
#[derive(Copy, Clone)]
pub struct nvoices(pub usize);

pub struct note((f32,f32));
#[derive(Copy, Clone)]
pub struct noteon(pub f32, pub f32);
#[derive(Copy, Clone)]
pub struct noteoff(pub u8);
#[derive(Copy, Clone)]
pub struct f0(pub f32);
// pub struct gain(f32);
