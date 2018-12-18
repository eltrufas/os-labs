extern crate ggez;
use self::ggez::*;
use self::ggez::graphics::{DrawMode, Point2};

use super::simulator;

pub struct State {
    sim: simulator::Simulator,
    frame: usize,
    pos_x: f32,
}

impl State {
    pub fn new(sim: simulator::Simulator) -> Self{
        State {
            sim: sim,
            frame: 0,
            pos_x: 0.0,
        }
    }

    pub fn run(&mut self) -> GameResult<()> {
        let c = conf::Conf::new();
        let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();

        event::run(ctx, self)
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()>{
        if self.frame % 16 == 0 {
            self.sim.run_frame();
        }

        self.pos_x = self.pos_x % 800.0 + 1.0;

        self.frame += 1;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::circle(ctx,
                         DrawMode::Fill,
                         Point2::new(self.pos_x, 380.0),
                         100.0,
                         2.0)?;
        graphics::present(ctx);
        Ok(())

    }
}

