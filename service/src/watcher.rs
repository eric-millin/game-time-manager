use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, sync::mpsc::Sender};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use windows::{Win32::Foundation::*, Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*};

pub fn check_procs_sync(sender: Sender<String>) {
    let poll_frequency = Duration::from_secs(5);
    let show_frequency = Duration::from_secs(15);

    let mut games: HashMap<String, Game> = HashMap::new();
    let mut last_shown: Option<Instant> = Default::default();

    let mut system = System::new_all();

    loop {
        thread::sleep(poll_frequency);

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
                // QUNS_AP probably fullscreen, but unlikely to be game
                QUNS_BUSY => (),
                QUNS_RUNNING_D3D_FULL_SCREEN => (),
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

        if system.process(pid).is_none() {
            continue;
        }

        let proc = system.process(pid).unwrap();

        println!("{}", proc.name());
        let name = proc.name();

        let game = games
            .entry(name.to_string())
            .or_insert_with(|| Game::new(name.to_string()));

        // if there's no session for this pid, create one, then update the session info
        let last_session = game.sessions.last_mut();
        if last_session.is_none() || last_session.unwrap().pid != pid.as_u32() {
            let session = Session {
                pid: pid.as_u32(),
                had_focus: vec![Focus(Some(Instant::now()), None)],
                run_time: 0,
            };

            game.sessions.push(session);
        }

        let session = game.sessions.last_mut().unwrap();
        session.run_time = proc.run_time();

        if last_shown.is_some() && last_shown.unwrap().elapsed() < show_frequency {
            continue;
        }

        last_shown = Some(Instant::now());

        let h = session.run_time / 60 / 60;
        let m = session.run_time / 60 % 60;

        match sender.send(format!("{}h {}m", h, m)) {
            Err(err) => println!("error on channel send: {:?}", err),
            _ => (),
        };
    }
}

struct Focus(/*start*/ Option<Instant>, /*end*/ Option<Instant>);

#[derive(Default)]
struct Session {
    pid: u32,
    had_focus: Vec<Focus>,
    run_time: u64,
}

#[derive(Default)]
struct Game {
    name: String,
    focus_time_total: u64,
    focus_time_7_days: u64,
    run_time_total: u64,
    run_time_7_days: u64,
    sessions: Vec<Session>, //RefCell<Vec<RefCell<Session<'a>>>>,
}

impl Game {
    fn new(name: String) -> Self {
        Self {
            name: name,
            focus_time_total: 0,
            focus_time_7_days: 0,
            run_time_total: 0,
            run_time_7_days: 0,
            sessions: Default::default(),
        }
    }
}
