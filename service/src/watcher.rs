use crate::config;
use crate::system_provider::SystemProvider;
use regex::RegexSet;
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, sync::mpsc::Sender};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

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

pub fn watch(sysprovider: &dyn SystemProvider, sender: Sender<String>) {
    let mut games: HashMap<String, Game> = HashMap::new();
    let mut last_shown: Option<Instant> = Default::default();
    let mut system = System::new_all();

    loop {
        let cfg = config::load().unwrap();

        thread::sleep(Duration::from_secs(cfg.watcher.poll_frequency));

        let game_proc;

        // try_get_game_pid could return Result<Option<Pid>, Error>, then use Some/None for control flow rather than Error
        //         match sysprovider.try_get_game_pid()? {
        match sysprovider.try_get_game_pid() {
            Ok(pid) => {
                system.refresh_processes();

                if system.process(pid).is_none() || std::process::id() == pid.as_u32() {
                    continue;
                }

                game_proc = system.process(pid).unwrap();
            }
            // in most cases this means no game-like process was found; TODO: add granular error handling
            _ => continue,
        }

        let game_exe_name = game_proc.name();
        let game_pid = game_proc.pid().as_u32();

        // TODO use wildcard checking, not regexp
        let set = RegexSet::new(cfg.watcher.ignore).unwrap();
        if set.matches(game_proc.name()).matched_any() {
            continue;
        }

        let game = games
            .entry(game_exe_name.to_string())
            .or_insert_with(|| Game::new(game_exe_name.to_string()));

        // if there's no session for this pid, create one, then update the session info
        let last_session = game.sessions.last_mut();
        if last_session.is_none() || last_session.unwrap().pid != game_pid {
            game.sessions.push(Session {
                pid: game_pid,
                focus_spans: vec![FocusSpan(Some(Instant::now()), None)],
                run_time: 0,
            });

            // set last shown to now so that the overlay isn't displayed until the next notification window
            last_shown = Some(Instant::now());
        }

        if last_shown.is_some()
            && last_shown.unwrap().elapsed()
                < Duration::from_secs(cfg.watcher.notification_frequency)
        {
            continue;
        }
        
        // TODO: use Clippy!
        
//         match last_shown {
//           Some(last_shown) => {
        // Do elapsed check
//             }
//            None => {
//             continue
//             }
//         }

        game.friendly_name =
            match sysprovider.try_get_product_name(game_proc.exe().display().to_string()) {
                Ok(name) => name,
                Err(_) => "".to_string(),
            };

        let session = game.sessions.last_mut().unwrap();
        session.run_time = game_proc.run_time();

        let h = session.run_time / 60 / 60;
        let m = session.run_time / 60 % 60;

        match sender.send(format!("{}h {}m", h, m)) {
            Ok(_) => {
                println!("sent message {:?} for {}", Instant::now(), game_proc.name());
                last_shown = Some(Instant::now());
            }
            Err(err) => println!("error on channel send: {:?}", err),
        };
    }
}
