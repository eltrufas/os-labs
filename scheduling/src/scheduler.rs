use std::collections::{VecDeque, HashMap};
use super::{Process, ProcessState};

pub trait Scheduler {
    fn add_proc(&mut self, &Process);
    fn run_frame(&mut self) -> HashMap<i32, ProcessState>;
}

pub struct RRScheduler {
    proc_queue: VecDeque<RRProcess>,
    current_proc: Option<RRProcess>,
    current_frame: usize,
    proc_timer: i32,
    r: i32,
}

#[derive(Clone)]
struct RRProcess {
    id: i32,
    time_left: usize,
    start_time: usize,
	wait_time: usize
}

impl RRScheduler {
    pub fn new(r: i32) -> Self {
        RRScheduler {
            proc_queue: VecDeque::new(),
            current_proc: None,
            current_frame: 0,
            proc_timer: r,
            r,
        }
    }
}

impl Scheduler for RRScheduler {
    fn add_proc(&mut self, p: &Process) {
        let rrp = RRProcess {
            id: p.id,
            time_left: p.duration,
            start_time: self.current_frame,
			wait_time: 0,
        };

        self.proc_queue.push_back(rrp);
    }

    fn run_frame(&mut self) -> HashMap<i32, ProcessState> {
        self.proc_timer -= 1;

        let mut frame_results = HashMap::new();

        let mut next_proc = self.current_proc.clone().map(|p| {
            debug!("Running process {} this frame, {} frames left", p.id, p.time_left);
            RRProcess {
                time_left: p.time_left - 1,
                ..p
            }
        }).and_then(|p| {
            if p.time_left <= 0 {
                frame_results.insert(p.id, ProcessState::Finished);
                None
            } else {
                Some(p)
            }
        });
        
        if self.proc_timer <= 0 {
            if let Some(p) = next_proc {
                self.proc_queue.push_back(p);
                next_proc = None;
            }
        }

        if let None = next_proc {
            debug!("Popping next process");
            next_proc = self.proc_queue.pop_front();
            self.proc_timer = self.r;
        }

		self.current_proc = next_proc;

        if let Some(ref p) = self.current_proc {
            frame_results.insert(p.id, ProcessState::Running);
        }

        for p in self.proc_queue.iter_mut() {
			p.wait_time += 1;
            frame_results.insert(p.id, ProcessState::Waiting(p.wait_time));
        }

        self.current_frame += 1;

        frame_results
    }
}

