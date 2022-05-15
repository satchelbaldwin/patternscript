use cgmath::{Deg, Vector2, Vector3};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color};
use ggez::{mint, timer};
use ggez::{Context, ContextBuilder, GameResult};

use patternscript::interpreter::entity::{Entity, Hitbox};
use patternscript::interpreter::evaluate::*;
use patternscript::interpreter::*;
use patternscript::parser::lexer::{Lexer, Token};
use patternscript::parser::parser::*;
use patternscript::parser::types::Op;
use std::env;
use std::fs;
use std::process;

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("patternscript gui", "")
        .build()
        .expect("failed to create window");

    let args = env::args().collect::<Vec<String>>();
    let mut world =
        Interpreter::from_parse_result(Parser::parse_from_file(args[1].clone()).unwrap()).unwrap();
    let e = Entity {
        position: Vector2 { x: 300.0, y: 20.0 },
        velocity: Vector2 { x: 0.0, y: 10.0 },
        rotation: Deg(90.0),
        speed: Some(50.0),
        lifetime: 600000,
        color: Vector3 {
            x: 255,
            y: 0,
            z: 255,
        },
        hitbox: Hitbox {
            size: Vector2 { x: 8, y: 8 },
            offset: Vector2 { x: 0.0, y: 0.0 },
            hitbox_type: entity::HitboxType::Rectangle,
        },
        behavior: entity::Behavior::Pattern(args[2].clone()),
        position_fn: None,
        velocity_fn: None,
        instance_vars: None,
    };
    world.spawn_direct(&e);

    let my_game = MyGame::new(&mut ctx, world);
    event::run(ctx, event_loop, my_game);
}

struct MyGame<'a> {
    world: Interpreter<'a>,
}

impl<'a> MyGame<'a> {
    pub fn new(_ctx: &mut Context, world: Interpreter<'a>) -> MyGame<'a> {
        // Load/create resources such as images here.
        MyGame { world }
    }
}

impl<'a> EventHandler for MyGame<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 120;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.world.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);

        for e in &self.world.entities {
            let color = e.entity.color;

            let color = graphics::Color {
                r: color[0] as f32 / 255.0,
                g: color[1] as f32 / 255.0,
                b: color[2] as f32 / 255.0,
                a: 1.0,
            };
            let rx = e.entity.position.x as f32 - (e.entity.hitbox.size.x / 2) as f32;
            let ry = e.entity.position.y as f32 - (e.entity.hitbox.size.y / 2) as f32;

            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(rx, ry, 8.0, 8.0),
                color,
            )?;

            if (rx < 1200. && rx > 0. && ry > 0. && ry < 800.) {
                graphics::draw(ctx, &rect, graphics::DrawParam::default())?;
            }
        }

        let fps = ggez::timer::fps(ctx);
        println!("{}", fps);
        println!("{}", self.world.entities.len());

        graphics::present(ctx)
    }
}
