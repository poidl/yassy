pub const NOSC: usize = 4;

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

#[derive(Copy, Clone)]
pub struct note2osc(pub [u8; NOSC]);

#[derive(Copy, Clone)]
pub struct maxnotes(pub u8);

pub struct note((f32,f32));

// f0, vel, voice index
#[derive(Copy, Clone)]
pub struct noteon(pub f32, pub f32, pub u8);
// note number, voice index
#[derive(Copy, Clone)]
pub struct noteoff(pub u8, pub u8);

#[derive(Copy, Clone)]
pub struct f0(pub f32);
// pub struct gain(f32);
