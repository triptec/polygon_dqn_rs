use geo::{LineString, GeometryCollection, Geometry, Point, Coordinate, Line};
use std::{fs};
use geojson::{GeoJson, quick_collection};
use rand::prelude::*;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use geo::algorithm::map_coords::MapCoordsInplace;
use geo::intersects::Intersects;
use num_traits::Float;
use geo::algorithm::euclidean_distance::EuclideanDistance;
use rayon::prelude::*;
extern crate sdl2;
extern crate geo;
mod ray;
mod renderer;
mod environment;
mod agent;
use sdl2::{rect, EventPump, pixels};
use line_intersection::{LineInterval, LineRelation};
use sdl2::render::WindowCanvas;
use crate::ray::Ray;
use crate::renderer::Renderer;
use crate::environment::Environment;
use crate::agent::Agent;

fn main() -> Result<(), String> {
    let min_distance_to_obstacle = 0.005;
    let mut last_distance_to_obstacle = 0.0;
    let mut _env = Environment::new();
    let mut agent = Agent::new(Point::new(0.45, 0.55), 2.0);
    let mut direction_change = 0.0;
    let mut renderer = Renderer::new(1000.0, 1000.0);

    agent.cast_rays();


    'running: loop {
        if renderer.handle_events() {
            break 'running
        }


        let distance = agent.rays.iter().map(|x| x.length).fold(1000.0, |a, b| a.min(b));

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

        renderer.render_rays(&agent.rays, Color::RGB(0, 0, 255));

        _env.update_state(&mut agent);

        renderer.render_rays(&agent.rays, Color::RGB(255, 0, 0));

        renderer.render_lines(&_env.lines, Color::RGB(0, 255, 0));

        renderer.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use line_intersection::{LineInterval, LineRelation};

    #[test]
    fn test_intersect2() {
        let line1 = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. });
        let line2 = Line::new(Coordinate { x: 1., y: 0. }, Coordinate { x: 0., y: 1. });
        let s1 = LineInterval::line_segment(line1);
        let s2 = LineInterval::line_segment(line2);
        let relation = LineRelation::DivergentIntersecting((0.5, 0.5).into());
        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
        let line1 = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. });
        let line2 = Line::new(Coordinate { x: -1., y: 0. }, Coordinate { x: 0., y: -1. });
        let s1 = LineInterval::line_segment(line1);
        let s2 = LineInterval::line_segment(line2);
        let relation = LineRelation::DivergentDisjoint;
        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));

    }

}