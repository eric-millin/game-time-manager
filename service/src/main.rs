#![windows_subsystem = "windows"]

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use nwg::NativeUi;

mod config;
mod overlay;
mod system_provider;
mod watcher;

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _ui = overlay::Overlay::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
