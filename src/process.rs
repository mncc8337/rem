use notify::{Event, EventKind, event::{AccessKind, AccessMode}, RecursiveMode, Result, Watcher};
use std::collections::HashSet;
use std::sync::mpsc;
use crate::config::ConfigManager;
use crate::entry::Entry;
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};
extern crate libnotify;

#[derive(Clone)]
struct Remind {
    entry: Entry,
    life_time: time::Duration,
}

pub struct Process {
    pub configman: ConfigManager,
    pending_reminds: Arc<Mutex<Vec<Remind>>>,
    kill_switch: Arc<Mutex<bool>>,
}

enum ConfigEvent {
    ConfigChanged,
    ConfigUnchanged,
}

impl Process {
    pub fn new(configman: ConfigManager) -> Self {
        Process {
            configman,
            pending_reminds: Arc::new(Mutex::new(Vec::<Remind>::new())),
            kill_switch: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(process: Arc<Mutex<Self>>) {
        let (watcher_tx, watcher_rx) = mpsc::channel::<Result<Event>>();
        let (config_tx, config_rx) = mpsc::channel::<ConfigEvent>();

        // set up file watcher
        let mut watcher = notify::recommended_watcher(watcher_tx).unwrap();

        {
            let proc = process.lock().unwrap();
            watcher.watch(&proc.configman.config_path, RecursiveMode::NonRecursive)
                .unwrap();
        }

        // set up thread for file watcher
        let proc_clone = Arc::clone(&process);
        thread::spawn(move || {
            println!("file watcher thread started");
            loop {
                {
                    let proc = proc_clone.lock().unwrap();
                    let ks = proc.kill_switch.lock().unwrap();
                    if *ks {
                        break;
                    }
                }
                match watcher_rx.recv_timeout(time::Duration::from_secs(1)) {
                    Ok(Ok(event)) => {
                        if let EventKind::Access(AccessKind::Close(AccessMode::Write)) = event.kind {
                            let mut proc = proc_clone.lock().unwrap();
                            println!("config file changed, reloading config ...");
                            let message: ConfigEvent;
                            if proc.reload_config() {
                                message = ConfigEvent::ConfigChanged;
                            } else {
                                message = ConfigEvent::ConfigUnchanged;
                            }
                            let _ = config_tx.send(message);
                        }
                    },
                    Ok(Err(e)) => println!("watch error: {:?}", e),
                    Err(mpsc::RecvTimeoutError::Timeout) => {},
                    Err(_) => break,
                }
            }

            println!("file watcher thread terminated");
        });

        // set up queue
        let mut rem_queue = std::collections::VecDeque::<Remind>::new();
        {
            let process = process.lock().unwrap();
            for entry in &process.configman.config.entries {
                rem_queue.push_back(Remind {
                    entry: entry.clone(),
                    life_time: time::Duration::from_secs(entry.interval),
                });
                println!("new remind '{}' added to queue", entry.name);
            }
        }
        
        // sort the queue
        let mut vec: Vec<_> = rem_queue.into_iter().collect();
        vec.sort_by(|a, b| { a.life_time.cmp(&b.life_time) });
        rem_queue = vec.into_iter().collect();

        let mut start_time = time::Instant::now();

        libnotify::init("rem").unwrap();

        println!("reminder thread started");
        'main_loop: loop {
            if rem_queue.is_empty() {
                let pending_len: usize;
                {
                    let proc = process.lock().unwrap();
                    let pending = proc.pending_reminds.lock().unwrap();
                    pending_len = pending.len();
                }
                if pending_len == 0 {
                    println!("no remind, waiting until config is changed");
                    loop {
                        match config_rx.recv_timeout(time::Duration::from_secs(2)) {
                            Ok(ConfigEvent::ConfigChanged) => break,
                            Ok(ConfigEvent::ConfigUnchanged) => {},
                            Err(mpsc::RecvTimeoutError::Timeout) => {},
                            Err(_) => break 'main_loop,
                        }
                    }

                    start_time = time::Instant::now();
                }
            }

            let elapsed = start_time.elapsed();

            {
                let proc = process.lock().unwrap();
                let ks = proc.kill_switch.lock().unwrap();
                if *ks {
                    break;
                }

                let mut pending = proc.pending_reminds.lock().unwrap();
                if pending.len() > 0 {
                    for remind in pending.iter_mut() {
                        let mut rm = remind.clone();
                        rm.life_time = time::Duration::from_secs(remind.entry.interval);
                        rm.life_time += elapsed;
                        rem_queue.push_back(rm);
                        println!("pending remind '{}' added to queue", remind.entry.name);
                    }
                    pending.clear();
                    // sort the queue
                    // i know this is very inefficient
                    // but it is the most simple
                    let mut vec: Vec<_> = rem_queue.into_iter().collect();
                    vec.sort_by(|a, b| { a.life_time.cmp(&b.life_time) });
                    rem_queue = vec.into_iter().collect();
                }
            }

            loop {
                if rem_queue.len() == 0 {
                    // goto the empty queue handling above
                    // this ensure that pop_front or get(0) always works
                    continue 'main_loop;
                }
                if elapsed < rem_queue.get(0).unwrap().life_time {
                    // the queue is sorted so we can ignore all other elements
                    break;
                }
                let remind = rem_queue.pop_front().unwrap();

                // create a new notification
                let mut icon = None;
                if remind.entry.icon != "" {
                    icon = Some(remind.entry.icon.as_str());
                }
                let notif = libnotify::Notification::new(
                    &remind.entry.name,
                    Some(remind.entry.message.as_str()),
                    icon
                );
                notif.set_category("rem");
                notif.set_app_name("rem");
                notif.set_timeout(0);
                match remind.entry.urgency {
                    0 => notif.set_urgency(libnotify::Urgency::Low),
                    2 => notif.set_urgency(libnotify::Urgency::Critical),
                    _ => notif.set_urgency(libnotify::Urgency::Normal),
                }
                notif.show().unwrap();
                println!("'{}' reminded", remind.entry.name);

                {
                    let proc = process.lock().unwrap();
                    // verify whether the entry is deleted or not
                    for entry in &proc.configman.config.entries {
                        if entry.creation_time == remind.entry.creation_time {
                            rem_queue.push_back(Remind {
                                entry: entry.clone(),
                                life_time: elapsed + time::Duration::from_secs(entry.interval),
                            });
                            println!("remind '{}' added back to queue", entry.name);
                            break;
                        }
                    }

                }
            }
            // sort the queue
            // this is inefficient too and can be improved
            let mut vec: Vec<_> = rem_queue.into_iter().collect();
            vec.sort_by(|a, b| { a.life_time.cmp(&b.life_time) });
            rem_queue = vec.into_iter().collect();

            // limit execution
            thread::sleep(time::Duration::from_millis(500));
        }

        println!("remind thread terminated");
    }

    pub fn terminate(&mut self) {
        let mut ks = self.kill_switch.lock().unwrap();
        *ks = true;
    }

    fn reload_config(&mut self) -> bool {
        let old_config = self.configman.config.entries.clone();
        let result = ConfigManager::open(self.configman.config_path.clone());
        let new_config: ConfigManager;
        match result {
            Err(e) => {
                println!("error while loading config: {:?}", e);
                self.terminate();
                return false;
            },
            Ok(c) => {
                new_config = c;
            },
        }
        
        let changed: bool;

        if old_config != new_config.config.entries {
            let hs_old: HashSet<_> = old_config.iter().cloned().collect();
            let hs_new: HashSet<_> = new_config.config.entries.iter().cloned().collect();

            let added_entry: Vec<Entry> = hs_new
                .difference(&hs_old)
                .cloned()
                .into_iter()
                .collect();

            let mut new_reminds = Vec::<Remind>::new();
            for i in 0..added_entry.len() {
                new_reminds.push(Remind {
                    entry: added_entry[i].clone(),
                    life_time: time::Duration::from_secs(0),
                });
            }

            changed = new_reminds.len() > 0;

            let mut pending = self.pending_reminds.lock().unwrap();
            pending.append(&mut new_reminds);
            self.configman = new_config;

        } else {
            changed = false;
        }

        if changed {
            println!("config reloaded");
        }
        changed
    }
}
