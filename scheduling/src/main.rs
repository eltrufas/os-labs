use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use relm::{Relm, Update, Widget};
use gtk::prelude::*;
use gtk::{Window, Inhibit, WindowType, Button};

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

mod process;
mod scheduler;
mod simulator;

#[derive(Debug, Deserialize)]
pub struct Process {
    pub id: i32,
    pub start_time: usize,
    pub duration: usize,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    Waiting(usize),
    Running,
    Finished,
    None,
}

struct Model {
    sim: simulator::Simulator,
}

#[derive(Msg)]
enum Msg {
    AdvanceFrame,
}

struct Win {
    model: Model,
    window: Window,
	proc_list: gtk::ListStore,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = simulator::Simulator;
    type Msg = Msg;

    fn model(_: &Relm<Self>, sim: simulator::Simulator) -> Model {
        Model {
            sim
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            AdvanceFrame => {
                self.model.sim.run_frame();
            }
        }
    }
}

fn append_column(tree: &gtk::TreeView, title: &str, id: i32) {
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
	column.set_title(title);
	tree.append_column(&column);
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Vertical);
        let open_button = Button::new_with_label("Open");
        let save_button = Button::new_with_label("Save");


        let list_model = gtk::ListStore::new(&[
            u32::static_type(),
            u32::static_type(),
            u32::static_type(),
            u32::static_type(),
        ]);

		for p in model.sim.procs.iter() {
			list_model.insert_with_values(
				None,
				&[0, 1, 2, 3],
				&[
					&(p.id as u32),
					&(p.start_time as u32),
					&(p.duration as u32),
					&(p.priority as u32),
				]);
		}

        let tree = gtk::TreeView::new();

    	tree.set_model(Some(&list_model));

        append_column(&tree, "id", 0);
    	append_column(&tree, "tiempo de inicio", 1);
    	append_column(&tree, "duracion", 2);
    	append_column(&tree, "prioridad", 3);


        button_box.pack_start(&open_button, false, false, 0);
        button_box.pack_start(&save_button, false, false, 0);
        hbox.pack_start(&button_box, false, false, 0);
        hbox.pack_start(&tree, true, true, 0);
        window.add(&hbox);
        window.set_title("Calendarización de procesos");
        window.show_all();
 

        Win {
            model,
            window,
			proc_list: list_model,

        }
    }

}

fn read_process_file(filename: &str) -> Result<Vec<Process>, Box<Error>> {
    let mut proc_file = File::open(filename)?;
    let mut contents = String::new();
    proc_file.read_to_string(&mut contents)?;

    let mut id_counter = 0;
    let processes: Vec<Process> = serde_json::from_str(&contents)?;

    let processes = processes.into_iter().map(|p: Process| {
        id_counter += 1;

        Process {
            id: id_counter, 
            ..p
        }
    }).collect();

    Ok(processes)
} 

fn main() {
	env_logger::init();

	info!("Starting up!");

    let procs = match read_process_file("processes.txt") {
        Ok(results) => results,
        Err(why) => panic!("No se pudo abrir el archivo: {}", why.description()),
    };

    let sched = scheduler::RRScheduler::new(2);

    let sim = simulator::Simulator::new(procs, sched);

    Win::run(sim).unwrap();
}

