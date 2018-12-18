use super::scheduler;
use super::{ProcessState, Process};

use std::time::Duration;
use std::thread;

pub struct Simulator {
    pub procs: Vec<Process>,
    sched: Box<dyn scheduler::Scheduler>,
    pub state_table: Vec<Vec<ProcessState>>,
    remaining_procs: usize,
    frame: usize,
}

impl Simulator {
    pub fn new<S: scheduler::Scheduler + 'static>(procs: Vec<Process>, sched: S) -> Self {
        debug!("Processes: {:?}", procs);
        Simulator {
            remaining_procs: procs.len(),
            procs,
            sched: Box::new(sched),
            state_table: Vec::new(),
            frame: 0,
        }
    }

    fn last_frame(&self) -> Vec<ProcessState> { 
        self.state_table.last().map(|s| s.clone()).unwrap_or_else(|| {
            vec![ProcessState::None; self.procs.len()]
        })
    }

    pub fn run_frame(&mut self) {
        debug!("Running frame {}", self.frame);
        for p in self.procs.iter() {
            if p.start_time == self.frame {
                debug!("Adding process {}", p.id);
                self.sched.add_proc(p);
            }
        }

        let mut results = self.sched.run_frame();
        debug!("Results: {:?}", results);

        let last_frame = self.last_frame();

        self.remaining_procs -= results.values().filter(|s| s == &&ProcessState::Finished).count();

        let next_frame: Vec<ProcessState> = self.procs.iter().zip(last_frame.iter()).map(|(p, ls)| {
            results.remove(&p.id).unwrap_or(ls.clone()) 
        }).collect();

        debug!("Remaining procs: {}", self.remaining_procs);

        self.state_table.push(next_frame);
        self.frame += 1;
    }

    pub fn run_all(&mut self) {
        while self.remaining_procs != 0 {
            self.run_frame();
        }
    } 
}

