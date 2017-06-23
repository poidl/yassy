extern crate libc;
extern crate midi;
extern crate lv2;

pub mod oscillator;
// pub mod voice;
pub mod lv2_plugin;
// pub mod synth;
pub mod utils;
// pub mod plugin;
// pub mod adsr;
pub mod observer;
pub mod types;

use std::ptr;
use std::mem;
use oscillator::*;

// have to define new type. Otherwise error: "cannot define inherent impl
// for a type outside of the crate where the type is defined; define and
// implement a trait or new type instead"
struct Descriptor(lv2::LV2Descriptor);

impl Descriptor {
    pub extern "C" fn instantiate(_descriptor: *const lv2::LV2Descriptor,
                                  fs: f64,
                                  _bundle_path: *const libc::c_char,
                                  hostfeatures: *const (*const lv2::LV2Feature))
                                  -> lv2::LV2Handle {
        unsafe {
        let mut buf1 = Box::new(0f32);
        let mut buf2 = Box::new(0f32);
        let mut b1 = &mut*buf1 as *mut f32;
        let mut b2 = &mut*buf2 as *mut f32;
        let mut osc = Box::new(OscBLIT::new(&mut*b1));
        let mut bx = Box::new(lv2_plugin::Lv2SynthPlugin::new(&mut*b2));
        let featureptr = lv2::mapfeature(hostfeatures, "http://lv2plug.in/ns/ext/urid#map");
        match featureptr {
            Ok(fp) => bx.map = fp as *mut lv2::LV2UridMap,
            _ => return ptr::null_mut(),
        }
        bx.seturis();

        let r1 = &mut*osc as *mut OscBLIT;
        bx.midiMessage.observers.push(&mut *r1);
        // plugin.fs.observers.push(&mut plugin.synth);

        // bx.set_fs(fs);

        bx.fs.observers.push(&mut *r1);
        bx.bufferpos.observers.push(&mut *r1);
        bx.fs.update(types::fs(fs));
        // println!{"********* a **********"}
        bx.bufferpos.update(0u32);
        // println!{"********* d **********"}
        // let mut bb2 = bx.in_port_synth as *mut f32;
        let mut bb1 = osc.buf as *mut f32;
        bx.in_port_synth = &mut *bb1;
        // bb2 = bb1;
        // println!{"********* G **********"}


        let ptr = (&*bx as *const lv2_plugin::Lv2SynthPlugin) as *mut libc::c_void;
        mem::forget(bx);
        mem::forget(osc);
        mem::forget(buf1);
        mem::forget(buf2);
        ptr
        }
    }

    pub extern "C" fn connect_port(handle: lv2::LV2Handle, port: u32, data: *mut libc::c_void) {
        let synth: *mut lv2_plugin::Lv2SynthPlugin = handle as *mut lv2_plugin::Lv2SynthPlugin;
        unsafe { (*synth).connect_port(port, data) }
    }
    pub extern "C" fn activate(_instance: lv2::LV2Handle) {}

    pub extern "C" fn run(instance: lv2::LV2Handle, n_samples: u32) {
        // println!{"*********** RUN "}
        unsafe {
            let synth = instance as *mut lv2_plugin::Lv2SynthPlugin;
            (*synth).run(n_samples)
        }
    }

    pub extern "C" fn deactivate(_instance: lv2::LV2Handle) {}

    pub extern "C" fn cleanup(instance: lv2::LV2Handle) {

        unsafe {
            let synth = instance as *mut lv2_plugin::Lv2SynthPlugin;
            // (*synth).cleanup();
            // ptr::read(instance as *mut Amp); // no need for this?
            libc::free(instance as lv2::LV2Handle)
        }
    }
    pub extern "C" fn extension_data(_uri: *const u8) -> (*const libc::c_void) {
        ptr::null()
    }
}

static S: &'static [u8] = b"http://example.org/yassy\0";

static mut DESC: lv2::LV2Descriptor = lv2::LV2Descriptor {
    uri: 0 as *const libc::c_char, // ptr::null() isn't const fn (yet)
    instantiate: Descriptor::instantiate,
    connect_port: Descriptor::connect_port,
    activate: Some(Descriptor::activate),
    run: Descriptor::run,
    deactivate: Some(Descriptor::deactivate),
    cleanup: Descriptor::cleanup,
    extension_data: Descriptor::extension_data,
};

#[no_mangle]
pub extern "C" fn lv2_descriptor(index: i32) -> *const lv2::LV2Descriptor {
    if index != 0 {
        return ptr::null();
    } else {
        // credits to ker on stackoverflow:
        // http://stackoverflow.com/questions/31334356 (duplicate) or
        // http://stackoverflow.com/questions/25880043
        let ptr = S.as_ptr() as *const libc::c_char;
        unsafe {
            DESC.uri = ptr;
            return &DESC as *const lv2::LV2Descriptor;
        }
    }
}
