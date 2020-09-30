#![feature(test)]
extern crate test;

use geo::Point;

use sdl2::pixels::Color;

use num_traits::Float;


extern crate geo;
extern crate sdl2;
mod agent;
mod env;
mod ray;
mod renderer;

use crate::agent::Agent;
use crate::env::Env;
use crate::renderer::Renderer;

fn main() -> Result<(), String> {
    let min_distance_to_obstacle = 0.005;
    let mut last_distance_to_obstacle = 0.0;
    let mut _env = Env::new();
    let mut agent = Agent::new(Point::new(0.45, 0.55), 2.0);
    let mut direction_change = 0.0;
    let mut renderer = Renderer::new(_env.scalex, _env.scaley);

    agent.cast_rays();

    'running: loop {
        if renderer.handle_events() {
            break 'running;
        }

        let distance = agent
            .rays
            .iter()
            .map(|x| x.length)
            .fold(1000.0, |a, b| a.min(b));

        if distance < min_distance_to_obstacle {
            agent.speed = 0.00005;
            if distance > last_distance_to_obstacle {
                last_distance_to_obstacle = distance;
                direction_change = 0.0;
            } else {
                last_distance_to_obstacle = distance;
                direction_change += rand::random::<f64>() - 0.5;
            }
        } else {
            agent.speed = 0.0004;
            direction_change = 0.0;
        }
        agent.step(direction_change);

        renderer.clear();

        renderer.render_line_strings(&_env.line_strings.iter().map(|ls| ls).collect(), Color::RGB(0, 255, 0));

        let culled = Env::find_culled(&agent.rays, &_env.line_strings, agent.position);
        renderer.render_line_strings(&culled, Color::RGB(255, 0, 255));

        renderer.render_rays(&agent.rays, Color::RGB(0, 0, 255));

        _env.update_state(&mut agent);

        renderer.render_rays(&agent.rays, Color::RGB(255, 0, 0));



        renderer.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
