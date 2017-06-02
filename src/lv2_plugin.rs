extern crate libc;
extern crate lv2;
use std::ffi::CString;
use std::ptr;
use plugin;
use midi;
use std::str;

pub struct Synthuris {
    pub midi_event: lv2::LV2Urid,
}

impl Synthuris {
    fn new() -> Synthuris {
        Synthuris { midi_event: 0 as lv2::LV2Urid }
    }
}

#[repr(C)]
pub struct Lv2SynthPlugin {
    pub map: *mut lv2::LV2UridMap,
    pub in_port: *const lv2::LV2AtomSequence,
    pub output: *mut f32,
    pub uris: Synthuris,
    pub plugin: plugin::SynthPlugin,
}

impl Lv2SynthPlugin {
    pub fn new() -> Lv2SynthPlugin {
        let lv2plugin = Lv2SynthPlugin {
            map: ptr::null_mut(),
            in_port: ptr::null(),
            output: ptr::null_mut(),
            uris: Synthuris::new(),
            plugin: plugin::SynthPlugin::new(),
        };
        lv2plugin
    }
    pub fn run(&mut self, n_samples: u32) {
        unsafe {
            let midievent = self.uris.midi_event;
            let sequence_iterator = (*self.in_port).into_iter();
            let mut i = 0;
            for ev in sequence_iterator.filter(|x| (*x).body.mytype == midievent) {
                let msg: &u8 = &*((ev as *const lv2::LV2AtomEvent).offset(1) as *const u8);
                self.plugin.midievent(msg as midi::MidiMessage);
                let ievent = (*ev).time_in_frames as u32;
                while i < ievent {
                    let amp = self.plugin.get_amp();
                    *self.output.offset(i as isize) = amp;
                    i =  i+1;
                }
            }
            while i < n_samples {
                let amp = self.plugin.get_amp();
                *self.output.offset(i as isize) = amp;
                i =  i+1;
            }
        }
    }
    pub fn seturis(&mut self) {
        unsafe {
            let s = "http://lv2plug.in/ns/ext/midi#MidiEvent";
            let cstr = CString::new(s).unwrap();
            let lv2_midi_midi_event = cstr.as_ptr();
            self.uris.midi_event = ((*self.map).map)((*self.map).handle, lv2_midi_midi_event);
        }
    }
    pub fn connect_port(&mut self, port: u32, data: *mut libc::c_void) {
        match port {
            0 => self.in_port = data as *const lv2::LV2AtomSequence,
            1 => self.output = data as *mut f32,
            _ => self.map_params(port, data),
        }
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.plugin.set_fs(fs);
    }
    pub fn get_amp(&mut self) -> f32 {
        self.plugin.get_amp()
    }
    fn map_params(&mut self, port: u32, data: *mut libc::c_void) {

        let nparams = self.plugin.params.len();
        let iport = port - 2; //TODO: don't hardcode number of input/output ports
        if iport <= nparams as u32 - 1 {
            println!("connecting port: {}", port);
            self.plugin.params[iport as usize] = data as *mut f32;
            // println!("param: {}",  *(self.synth.params[0]));
        } else {
            panic!("Not a valid PortIndex: {}", iport)
        }
    }
    pub fn cleanup(&mut self) {
        self.plugin.cleanup();
    }
}
