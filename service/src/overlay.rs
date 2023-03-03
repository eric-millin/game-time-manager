extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use std::thread;
use std::{sync::mpsc, time::Duration};

use nwd::NwgUi;
use std::sync::{Arc, Mutex};
use windows::Win32::{
    Graphics::Gdi::{GetMonitorInfoA, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY},
    UI::WindowsAndMessaging::GetForegroundWindow,
};

use crate::{config, watcher};

//use winapi::
// The overlay code is adapted from https://github.com/jcdavis/hroverlay, which is released under the
// Apache 2.0 license (https://raw.githubusercontent.com/jcdavis/hroverlay/main/LICENSE).

#[derive(Default, NwgUi)]
pub struct Overlay {
    #[nwg_control(size: (200, 100), position: (200, 0), flags: "POPUP", ex_flags: winapi::um::winuser::WS_EX_TOPMOST|winapi::um::winuser::WS_EX_LAYERED)]
    #[nwg_events( OnInit: [Overlay::on_init], OnWindowClose: [Overlay::close] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, margin: [0,0,0,0], spacing: 0)]
    layout: nwg::GridLayout,

    #[nwg_resource(family: "Arial", size: 100, weight: 700)]
    font: nwg::Font,

    #[nwg_control(text: "", size: (200, 100), font: Some(&data.font), h_align: HTextAlign::Right, background_color: Some([255, 0, 0]))]
    #[nwg_layout_item(layout: layout, row: 0, col: 0)]
    time_label: nwg::Label,

    #[nwg_control]
    #[nwg_events(OnNotice: [Overlay::on_notice])]
    notice: nwg::Notice,

    text: Arc<Mutex<String>>,
}

impl Overlay {
    fn on_init(&self) {
        use winapi::um::wingdi::RGB;
        use winapi::um::winuser::{SetLayeredWindowAttributes, LWA_COLORKEY};

        let notice = self.notice.sender();
        let (sender, receiver) = mpsc::channel();

        thread::spawn(|| {
            watcher::watch_procs(sender);
        });

        let display_text = self.text.clone();

        thread::spawn(move || {
            for rcv in receiver {
                let cfg = match config::load() {
                    Ok(c) => c,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };

                *display_text.lock().unwrap() = rcv;
                notice.notice();

                thread::sleep(Duration::from_secs(cfg.overlay.show_duration));

                *display_text.lock().unwrap() = "".to_string();
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

    fn on_notice(&self) {
        let cfg = match config::load() {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        match self.text.lock().unwrap().as_str() {
            "" => {
                self.window.set_visible(false);
            }
            text => {
                // TODO: font
                // nwg::Font::set_global_family(cfg.overlay.font.as_str())
                //     .expect("Failed to set default font");

                self.time_label.set_text(text);

                let (x, y) = get_right_corner();
                self.window.set_size(cfg.overlay.width, cfg.overlay.height);
                self.window.set_position(x - (cfg.overlay.width as i32), y);
                self.window.set_visible(true);
            }
        }
    }

    fn close(&self) {
        nwg::stop_thread_dispatch()
    }
}

fn get_right_corner() -> (i32, i32) {
    let mut minf = MONITORINFO::default();
    minf.cbSize = std::mem::size_of::<MONITORINFO>() as _;

    // https://github.com/ACK72/THRM-EX/blob/83588464c031082735b5f10bac881ef3d3d16d20/src/main.rs
    unsafe {
        let hwnd = GetForegroundWindow();
        let hmnt = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
        let _res = GetMonitorInfoA(hmnt, &mut minf as *mut MONITORINFO);
    }

    return (minf.rcMonitor.right, minf.rcMonitor.top);
}
