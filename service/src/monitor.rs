use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, sync::mpsc::Sender};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use windows::{Win32::Foundation::*, Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*};

pub fn check_procs_sync(sender: Sender<String>) {
    let mut games: HashMap<String, Game> = HashMap::new();
    let poll_frequency = Duration::from_secs(2);
    let mut s = System::new_all();

    loop {
        match sender.send("".to_string()) {
            Ok(_) => (),
            Err(err) => println!("error on channel send: {:?}", err),
        };

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

        // I guess what I want is linked list of start, finish, and PID;
        // time should reflect how much time this pid is in focus;
        // then use pid to check if this is the same session;

        s.refresh_processes();

        if s.process(pid).is_none() {
            continue;
        }

        let proc = s.process(pid).unwrap();

        println!("{}", proc.name());
        let name = proc.name();

        if !games.contains_key(name) {
            let g = Game {
                name: name.to_string(),
                focus_time_total: 0,
                focus_time_7_days: 0,
                run_time_total: 0,
                run_time_7_days: 0,
                sessions: Default::default(),
            };

            games.insert(name.to_string(), g);
        }

        // if there's no session for this pid, create one, then update the session info
        let mut sessions = games.get(name).unwrap().sessions.borrow_mut();

        // create a new session if needed
        if sessions.last().is_none() || sessions.last().unwrap().borrow().pid != pid.as_u32() {
            let session = Session {
                pid: pid.as_u32(),
                had_focus: RefCell::new(vec![Focus(Some(Instant::now()), None)]),
                run_time: 0,
            };

            sessions.push(RefCell::new(session));
        }

        let last = sessions.last();
        let mut current_session = last.unwrap().borrow_mut();
        current_session.run_time = proc.run_time();

        let h = current_session.run_time / 60 / 60;
        let m = current_session.run_time / 60 % 60;

        match sender.send(format!("{}h {}m", h, m)) {
            Ok(_) => (),
            Err(err) => println!("error on channel send: {:?}", err),
        };

        thread::sleep(Duration::from_secs(10));
    }
}

struct Focus<'a>(
    /*start*/ Option<Instant>,
    /*end*/ Option<&'a mut Instant>,
);

#[derive(Default)]
struct Session<'a> {
    pid: u32,
    had_focus: RefCell<Vec<Focus<'a>>>,
    run_time: u64,
}

#[derive(Default)]
struct Game<'a> {
    name: String,
    focus_time_total: u64,
    focus_time_7_days: u64,
    run_time_total: u64,
    run_time_7_days: u64,
    sessions: RefCell<Vec<RefCell<Session<'a>>>>,
}
