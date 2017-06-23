pub type MidiMessage<'a> = &'a [u8;3];

pub enum CcKind {
    ChannelVolume,
    Unknown
}

pub trait MidiTranslate {
    fn noteon(&self) -> bool;
    fn noteoff(&self) -> bool;

    fn f0(&self) -> f32;
    fn note_number(&self) -> u8;
    fn vel(&self) -> f32;

    // fn cc(&self) -> bool;
    // fn cc_type(&self) -> CcKind;
    // fn cc_value(&self) -> f32;
    // fn ccnr(&self) -> u8;
    // fn ccval(&self) -> u8;
}

impl<'a> MidiTranslate for MidiMessage<'a> {
    fn noteon(&self) -> bool {
        self[0] & 0xf0 == 0x90
    }
    fn noteoff(&self) -> bool {
        self[0] & 0xf0 == 0x80
    }
    fn f0(&self) -> f32 {
        let i = self[1];
        let f0 = (2.0f32.powf((((i as i8)-57) as f32)/12.0))*220.0;
        return f0
    }
    fn note_number(&self) -> u8 {
        self[1]
    }
    fn vel(&self) -> f32 {
        let i = self[2];
        return i as f32 / 127 as f32
    }
    // fn cc(&self) -> bool {
    //     self[0] & 0xf0 == 0xb0
    // }
    // fn cc_type(&self) -> CcKind {
    //     let x = self[1];
    //     match x {
    //         0x07 => return CcKind::ChannelVolume,
    //         _    => return CcKind::Unknown
    //     }
    // }
    // fn cc_value(&self) -> f32 {
    //         let x = self[2];
    //         (x as f32)/127f32
    // }
    // fn ccnr(&self) -> u8 {
    //     self[1]
    // }
    // fn ccval(&self) -> u8 {
    //     self[2]
    // }
}
