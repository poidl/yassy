extern crate libc;
extern crate lv2;
use std::ffi::CString;
use std::ptr;
use plugin;
use midi;
// use midi::*;
use std::str;

pub struct Synthuris {
    pub midi_event: lv2::LV2Urid,
    pub time_position_event: lv2::LV2Urid
}

impl Synthuris {
    fn new() -> Synthuris {
        Synthuris { 
            midi_event: 0 as lv2::LV2Urid, 
            time_position_event: 1 as lv2::LV2Urid
        }
    }
}

#[repr(C)]
pub struct Lv2SynthPlugin {
    pub map: *mut lv2::LV2UridMap,
    pub in_port: *const lv2::LV2AtomSequence,
    pub output: *mut f32,
    pub in_port_time: *const lv2::LV2AtomSequence,
    pub uris: Synthuris,
    pub plugin: plugin::SynthPlugin,
}

impl Lv2SynthPlugin {
    pub fn new() -> Lv2SynthPlugin {
        let lv2plugin = Lv2SynthPlugin {
            map: ptr::null_mut(),
            in_port: ptr::null(),
            output: ptr::null_mut(),
            in_port_time: ptr::null(),
            uris: Synthuris::new(),
            plugin: plugin::SynthPlugin::new(),
        };
        lv2plugin
    }
    pub fn run(&mut self, n_samples: u32) {
        unsafe {
            let midievent = self.uris.midi_event;
            let sequence_iterator = (*self.in_port).into_iter();

            // connect plugin
            self.plugin.audio_out = self.output;

            // filter midi events from atom sequence
            let midievent_iterator = sequence_iterator
                .filter(|x| (*x).body.mytype == midievent);

            // create an iterator with tuple items (ievent, midimessage)
            let ievent_midimessage = midievent_iterator
                .map(|x| {
                    (
                        (*x).time_in_frames as u32,
                        &*((x as *const lv2::LV2AtomEvent).offset(1) as *const [u8;3]) as midi::MidiMessage
                    )
                });
            
            // dispatch to plugin
            self.plugin.update(ievent_midimessage, n_samples);


            // let timepositionevent = self.uris.time_position_event;
            // // filter time_position events from atom sequence
            // let sequence_iterator2 = (*self.in_port_time).into_iter();
            // let timeposition_iterator = sequence_iterator2
            //     .filter(|x| (*(&(*x).body as *const lv2::LV2Atom as *const lv2::LV2AtomObject)).body.otype == timepositionevent);
            // for ev in timeposition_iterator {
            //     println!("************** TIMEPOS EVENT **************")
            // }
            

        }
    }
    pub fn seturis(&mut self) {
        unsafe {
            let mut s = "http://lv2plug.in/ns/ext/midi#MidiEvent";
            let mut cstr = CString::new(s).unwrap();
            let mut ptr = cstr.as_ptr();
            self.uris.midi_event = ((*self.map).map)((*self.map).handle, ptr);
            s = "http://lv2plug.in/ns/ext/time#Position";
            cstr = CString::new(s).unwrap();
            ptr = cstr.as_ptr();
            self.uris.time_position_event = ((*self.map).map)((*self.map).handle, ptr);
        }
    }
    pub fn connect_port(&mut self, port: u32, data: *mut libc::c_void) {
        match port {
            0 => self.in_port = data as *const lv2::LV2AtomSequence,
            1 => self.output = data as *mut f32,
            2 => self.in_port_time = data as *const lv2::LV2AtomSequence,
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
        let iport = port - 3; //TODO: don't hardcode number of input/output ports
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
