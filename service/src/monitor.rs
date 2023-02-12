//#![allow(clippy::single_match)]

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use windows::{
    core::*, Data::Xml::Dom::*, Win32::Foundation::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

pub fn check_procs_sync(sender: Sender<String>) {
    let dur = Duration::from_secs(60);

    let ignore_procs = vec!["GameOverlayUI.exe", "steamwebhelper.exe"];
    let mut s = System::new_all();
    let mut proc_pid: HashMap<String, Pid> = HashMap::new();
    let mut proc_start: HashMap<Pid, SystemTime> = HashMap::new();
    let mut last_shown = Instant::now();

    loop {
        s.refresh_processes();

        for steam_proc in s.processes_by_exact_name("steam.exe") {
            for (pid, proc) in s.processes() {
                if proc.parent() == Some(steam_proc.pid()) && !ignore_procs.contains(&proc.name()) {
                    proc_pid.entry(proc.name().to_string()).or_insert(*pid);

                    if !proc_start.contains_key(pid) {
                        proc_start.insert(*pid, SystemTime::now());
                        println!("updated map for {} {}", pid, proc.name());
                    } else {
                        let time = proc_start.get(pid);
                        let max = 2 * 60 * 60; // 2 hours
                        let allowed = Duration::from_secs(max);

                        match time.expect("time should be valid").elapsed() {
                            Err(e) => println!("unexpected error: {}", e.to_string()),
                            Ok(used) => {
                                if last_shown.elapsed() >= Duration::from_secs(15 * 60) {
                                    sender.send("15 minutes".to_string());
                                    println!("15 minutes");
                                    last_shown = Instant::now();
                                }

                                if used >= allowed {
                                    println!("DIE!");
                                    proc.kill();
                                }
                            }
                        };
                    }
                }
            }
        }

        thread::sleep(dur);
    }
}
