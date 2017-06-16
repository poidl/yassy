extern crate libc;
extern crate midi;
extern crate lv2;

pub mod oscillator;
pub mod voice;
pub mod lv2_plugin;
pub mod synth;
pub mod utils;
pub mod plugin;
pub mod adsr;

use std::ptr;
use std::mem;

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

        let mut bx = Box::new(lv2_plugin::Lv2SynthPlugin::new());
        let featureptr = lv2::mapfeature(hostfeatures, "http://lv2plug.in/ns/ext/urid#map");
        match featureptr {
            Ok(fp) => bx.map = fp as *mut lv2::LV2UridMap,
            _ => return ptr::null_mut(),
        }
        bx.seturis();
        bx.set_fs(fs);
        let ptr = (&*bx as *const lv2_plugin::Lv2SynthPlugin) as *mut libc::c_void;
        mem::forget(bx);
        ptr
    }

    pub extern "C" fn connect_port(handle: lv2::LV2Handle, port: u32, data: *mut libc::c_void) {
        let synth: *mut lv2_plugin::Lv2SynthPlugin = handle as *mut lv2_plugin::Lv2SynthPlugin;
        unsafe { (*synth).connect_port(port, data) }
    }
    pub extern "C" fn activate(_instance: lv2::LV2Handle) {}

    pub extern "C" fn run(instance: lv2::LV2Handle, n_samples: u32) {
        unsafe {
            let synth = instance as *mut lv2_plugin::Lv2SynthPlugin;
            (*synth).run(n_samples)
        }
    }

    pub extern "C" fn deactivate(_instance: lv2::LV2Handle) {}

    pub extern "C" fn cleanup(instance: lv2::LV2Handle) {

        unsafe {
            let synth = instance as *mut lv2_plugin::Lv2SynthPlugin;
            (*synth).cleanup();
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
