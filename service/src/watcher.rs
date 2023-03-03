use crate::config;
use lazy_static::lazy_static;
use regex::Regex;
use regex::RegexSet;
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, sync::mpsc::Sender};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Com::CoInitialize;
use windows::{
    Win32::Foundation::*, Win32::Storage::EnhancedStorage::*, Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*,
};

struct FocusSpan(/*start*/ Option<Instant>, /*end*/ Option<Instant>);

#[derive(Default)]
struct Session {
    pid: u32,
    focus_spans: Vec<FocusSpan>,
    run_time: u64,
}

#[derive(Default)]
struct Game {
    name: String,
    friendly_name: String,
    focus_time_total: u64,
    focus_time_7_days: u64,
    run_time_total: u64,
    run_time_7_days: u64,
    sessions: Vec<Session>,
}

impl Game {
    fn new(name: String) -> Self {
        let friendly_name = name.clone();
        Self {
            name: name,
            friendly_name: friendly_name,
            focus_time_total: 0,
            focus_time_7_days: 0,
            run_time_total: 0,
            run_time_7_days: 0,
            sessions: Default::default(),
        }
    }
}

pub fn watch_procs(sender: Sender<String>) {
    unsafe {
        // TODO: handle error
        CoInitialize(None);
    }

    let mut games: HashMap<String, Game> = HashMap::new();
    let mut last_shown: Option<Instant> = Default::default();
    let mut system = System::new_all();

    loop {
        let cfg = config::load().unwrap();

        thread::sleep(Duration::from_secs(cfg.watcher.poll_frequency));

        let pid;
        let res = unsafe { SHQueryUserNotificationState() };

        match res {
            Err(err) => println!("SHQueryUserNotificationState failed: {}", err),
            Ok(state) => match state {
                // QUNS_NOT_PRESENT - not fullscreen
                // QUNS_BUSY – fullscreen from F11 or video game
                // QUNS_RUNNING_D3D_FULL_SCREEN – fullscreen (Direct3D application is running in exclusive mode, i.e. fullscreen), uncommon, but likely a game
                // QUNS_PRESENTATION_MODE – fullscreen (a special mode for showing presentations, which are fullscreen), but unlikely to be game
                // QUNS_ACCEPTS_NOTIFICATIONS – not fullscreen
                // QUNS_QUIET_TIME – not fullscreen
                // QUNS_AP - probably fullscreen, but unlikely to be game
                // QUNS_RUNNING_D3D_FULL_SCREEN - definitely game, but adding overlay requires intercepting calls during rendering
                QUNS_BUSY => (),
                _ => continue,
            },
        }

        unsafe {
            let id: *mut u32 = &mut 0;

            let r = GetWindowThreadProcessId(GetForegroundWindow(), Some(id));
            if r == 0 {
                println!("win32 error: {:?}", GetLastError());
                continue;
            }

            pid = Pid::from_u32(*id);
        };

        system.refresh_processes();

        if system.process(pid).is_none() || std::process::id() == pid.as_u32() {
            continue;
        }

        let proc = system.process(pid).unwrap();
        let name = proc.name();

        // TODO Handle error!
        let set = RegexSet::new(cfg.watcher.ignore).unwrap();
        if set.matches(proc.name()).matched_any() {
            continue;
        }

        let game = games
            .entry(name.to_string())
            .or_insert_with(|| Game::new(name.to_string()));

        // if there's no session for this pid, create one, then update the session info
        let last_session = game.sessions.last_mut();
        if last_session.is_none() || last_session.unwrap().pid != pid.as_u32() {
            let session = Session {
                pid: pid.as_u32(),
                focus_spans: vec![FocusSpan(Some(Instant::now()), None)],
                run_time: 0,
            };
            game.sessions.push(session);

            // set last shown to now so that the overlay isn't displayed until the next notification window
            last_shown = Some(Instant::now());
        }

        if last_shown.is_some()
            && last_shown.unwrap().elapsed()
                < Duration::from_secs(cfg.watcher.notification_frequency)
        {
            continue;
        }

        unsafe {
            // TODO: error handling!
            let shi: IShellItem2 =
                SHCreateItemFromParsingName(PCWSTR(HSTRING::from(proc.exe()).as_ptr()), None)
                    .unwrap();
            game.friendly_name = match shi.GetString(&PKEY_Software_ProductName as _) {
                Ok(desc) => desc.to_string().unwrap(),
                Err(_) => "".to_string(),
            }
        };

        let session = game.sessions.last_mut().unwrap();
        session.run_time = proc.run_time();

        let h = session.run_time / 60 / 60;
        let m = session.run_time / 60 % 60;

        match sender.send(format!("{}h {}m", h, m)) {
            Ok(_) => {
                println!("sent message {:?} for {}", Instant::now(), proc.name());
                last_shown = Some(Instant::now());
            }
            Err(err) => println!("error on channel send: {:?}", err),
        };
    }
}
