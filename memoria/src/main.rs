#![feature(drain_filter)]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone)]
struct MemorySegment {
    id: String,
    address: usize,
    size: usize,
    free: bool,
}

#[derive(Deserialize, Clone)]
struct Process {
    id: String,
    size: usize,
    start_time: usize,
    duration: usize,
}

struct MemoryIndex(Vec<MemorySegment>);

impl MemoryIndex {
    fn new(total_mem: usize) -> MemoryIndex {
        let mut segments = Vec::with_capacity(100);

        segments.push(MemorySegment{
            id: String::new(),
            address: 0,
            size: total_mem,
            free: true,
        });

        MemoryIndex(segments)
    }

    fn allocate(&mut self, id: &str, size: usize) -> bool {
        let MemoryIndex(segments) = self;

        let r = segments.iter().enumerate()
            .filter(|(_, seg)| seg.free && seg.size >= size)
            .min_by_key(|(_, seg)| seg.size)
            .map(|(i, seg)| (i, seg.clone()));

        if let Some((i, seg)) = r {
            let frag = seg.size - size;
            let new_free_addr = seg.address + size;

            segments.insert(i, MemorySegment {
                id: String::from(id),
                address: seg.address,
                free: false,
                size: size,
            });

            if frag > 0 {
                segments[i + 1] = MemorySegment {
                    address: new_free_addr,
                    size: frag,
                    ..segments[i + 1].clone()
                };
            } else {
                segments.remove(i + 1);
            }

            true
        } else {
            false
        }
    }

    fn free(&mut self, id: &str) -> bool {
        let MemoryIndex(segments) = self;

        let r = segments.iter().enumerate()
            .find(|(_, seg)| seg.id == id);

        if let Some((i, _)) = r {
            segments[i].free = true;
            if i < segments.len() - 1 && segments[i + 1].free {
                segments[i].size += segments[i + 1].size;
                segments.remove(i + 1);
            }
            if i > 0 && segments[i - 1].free {
                segments[i - 1].size += segments[i].size;
                segments.remove(i);
            }

            true
        } else {
            false
        }
    }

    fn print(&self) {
        let MemoryIndex(segments) = self;

        let p = segments.iter()
            .map(|s| {
                if s.free {
                    format!("{:8} |{:14} libres|\n", s.address, s.size)
                } else {
                    format!("{:8} |{:10}|{:10}|\n", s.address, s.id, s.size)
                }
            })
            .collect::<Vec<String>>()
            .join("         -----------------------\n");


        println!("         -----------------------");
        println!("{}         -----------------------", p);
    }
}

struct Simulator {
    processes: Vec<Process>,
    started: Vec<(Option<usize>, Process)>,
    frame: usize,
    index: MemoryIndex,
}

impl Simulator {
    fn new(ps: Vec<Process>, index: MemoryIndex) -> Simulator {
        Simulator {
            started: Vec::with_capacity(ps.len()),
            processes: ps,
            index,
            frame: 0,
        }
    }

    fn run_frame(&mut self) {
        let frame = self.frame;
        let to_run = self.processes
            .drain_filter(|p| p.start_time <= frame)
            .map(|p| (None, p));

        self.started.extend(to_run);

        self.started = self.started.clone().into_iter()
            .filter_map(|(start, p)| {
                match start {
                    None  => {
                        if self.index.allocate(&p.id, p.size) {
                            println!("Reservando memoria para {}", p.id);
                            Some((Some(frame), p))
                        } else {
                            Some((None, p))
                        }
                    },
                    Some(start) => {
                        if start + p.duration < frame && self.index.free(&p.id) {
                            println!("Liberando memoria de {}", p.id);
                            None
                        } else {
                            Some((Some(start), p))
                        }
                    }
                }
            })
            .collect();

        self.frame += 1
    }

    fn run_all(&mut self) {
        while !self.processes.is_empty() || !self.started.is_empty() {
            println!("Instante {}", self.frame);
            self.run_frame();
            self.index.print();

            let frame = self.frame;
            let remaining: String = self.started.iter()
                .filter_map(|(i, p)| i.map(|s| (s, p)))
                .map(|(i, p)| {
                    format!("{}: {}", p.id, p.duration + 1 - (frame - 1 - i))
                }).collect::<Vec<String>>().join(", ");

            println!("Tiempos restantes: {}", remaining);
        }
    }
}

fn load_processes(filename: &str) -> Result<Vec<Process>, Box<Error>> {
    let mut proc_file = File::open(filename)?;
    let mut contents = String::new();
    proc_file.read_to_string(&mut contents)?;

    Ok(serde_json::from_str(&contents)?)
}

fn main() -> Result<(), Box<Error>> {
    let index = MemoryIndex::new(256);
    let processes = load_processes("processes.json")?;
    let mut sim  = Simulator::new(processes, index);

    sim.run_all();



    Ok(())
}
