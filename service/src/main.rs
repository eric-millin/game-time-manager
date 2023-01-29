use futures::executor::block_on;
use futures::StreamExt;
use futures_ticker::Ticker;
use std::collections::HashMap;
use std::time::{Duration,  SystemTime};
use sysinfo::{Pid, ProcessExt, System, SystemExt};

fn main() {
    block_on(check_procs())

    // https://github.com/LovecraftianHorror/vdf-rs/tree/main/keyvalues-serde
    // /mnt/c/Program\ Files\ \(x86\)/Steam/SteamApps/libraryfolders.vdf
    // C:\Program Files (x86)\Steam\SteamApps\libraryfolders.vdf
}

// ISteamApps/GetAppList
// api/appdetails?appids=

async fn check_procs() {
    let dur = Duration::from_secs(60);
    let mut tick = Ticker::new(dur);

    let ignore_procs = vec!["GameOverlayUI.exe", "steamwebhelper.exe"];
    let mut s = System::new_all();
    let mut proc_start: HashMap<Pid, SystemTime> = HashMap::new();
    //let mut i = 0;

    //let t = tick.next_tick();
    loop {
        // }
        s.refresh_all();

        for steam_proc in s.processes_by_exact_name("steam.exe") {
            for (pid, proc) in s.processes() {
                if proc.parent() == Some(steam_proc.pid()) && !ignore_procs.contains(&proc.name()) {
                    if !proc_start.contains_key(pid) {
                        proc_start.insert(*pid, SystemTime::now());
                        println!("updated map for {} {}", pid, proc.name());
                    } else {
                        let time = proc_start.get(pid).unwrap();
                        let max = 2 * 60 * 60; // 2 hours
                        let allowed = Duration::from_secs(max);

                        if time.elapsed().unwrap() >= allowed {
                            println!("DIE!");
                            proc.kill();
                        }
                    }
                }
            }
        }

        // i+=1;
        // if i >6 {
        //     break
        // }

        tick.next().await;
    }
}
