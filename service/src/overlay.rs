extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use nwd::NwgUi;
use std::sync::{Arc, Mutex};
use winapi::um::winuser::{WS_EX_LAYERED, WS_EX_TOPMOST};

//mod monitor;

// The overlay code is adapted from https://github.com/jcdavis/hroverlay, which is released under the
// Apache 2.0 license (https://raw.githubusercontent.com/jcdavis/hroverlay/main/LICENSE).

#[derive(Default, NwgUi)]
pub struct Overlay {
    #[nwg_control(size: (100, 100), position: (300, 300), flags: "POPUP|VISIBLE", ex_flags: WS_EX_TOPMOST|WS_EX_LAYERED)]
    #[nwg_events( OnInit: [Overlay::on_init], OnWindowClose: [Overlay::close] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, margin: [0,0,0,0], spacing: 0)]
    layout: nwg::GridLayout,

    #[nwg_resource(family: "Arial", size: 50, weight: 700)]
    font: nwg::Font,

    #[nwg_control(text: "--", size: (100, 120), font: Some(&data.font), h_align: HTextAlign::Right, background_color: Some([255, 0, 0]))]
    #[nwg_layout_item(layout: layout, row: 0, col: 0)]
    time_label: nwg::Label,

    #[nwg_control]
    #[nwg_events(OnNotice: [Overlay::draw_hr])]
    notice: nwg::Notice,

    text: Arc<Mutex<String>>,
}

impl Overlay {
    fn on_init(&self) {
        use winapi::um::wingdi::RGB;
        use winapi::um::winuser::{SetLayeredWindowAttributes, LWA_COLORKEY};

        let notice = self.notice.sender();

        let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

        thread::spawn(|| {
            create_dummy_updater(sender);
            //  monitor::check_procs_sync(sender);
        });

        let c = self.text.clone();

        thread::spawn(move || {
            for rcv in receiver {
                let mut text = c.lock().unwrap();
                *text = rcv;
                notice.notice();
            }
        });

        match self.window.handle {
            nwg::ControlHandle::Hwnd(hwnd) => unsafe {
                SetLayeredWindowAttributes(hwnd, RGB(255, 0, 0), 0, LWA_COLORKEY);
            },
            _ => {
                panic!("Bad handle type for window!")
            }
        }
    }

    fn draw_hr(&self) {
        // set new string atomically from updater
        match self.text.lock().unwrap().as_str() {
            "" => {
                // hide window in this case?
                self.time_label.set_text("--");
            }
            text => {
                self.time_label.set_text(text);
            }
        }

        // can wait and then hide after showing here, or fire async on timer to do the hide??
    }

    fn close(&self) {
        nwg::stop_thread_dispatch()
    }
}

// For basic UI testing. Just throws a bunch of data in the 0-199 range as a placeholder
fn create_dummy_updater(sender: Sender<String>) {
    let mut count: i64 = 0;
    loop {
        let r = count % 200;
        let s = format!("{}", r);
        sender.send(s);
        thread::sleep(Duration::from_millis(100));
        count += 1;
    }
}
