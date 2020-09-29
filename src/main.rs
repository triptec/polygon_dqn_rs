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
pub struct Environment {
    pub lines: Vec<LineString<f64>>
}
use rayon::prelude::*;
pub struct Ray {
    pub angle: f64,
    pub length: f64,
    pub max_length: f64,
    pub player_angle: f64,
    pub line_string: LineString<f64>,
}

impl Ray {
    pub fn new(angle: f64, length: f64, player_angle: f64, player_position: Point<f64>) -> Ray {
        Ray {
            angle,
            length,
            max_length: 0.0,
            player_angle,
            line_string: LineString(vec![
                Coordinate {x: player_position.x(), y: player_position.y()},
                Coordinate {x: player_position.x() + length * (player_angle + angle).cos(), y: player_position.y() + length * (player_angle + angle).sin()},
            ]),
        }
    }
}

pub struct Agent {
    pub speed: f64,
    pub position: Point<f64>,
    pub direction: f64,
    pub ray_count: f64,
    pub fov: f64,
    pub visibility: f64,
    pub rays: Vec<Ray>,
}

impl Agent {
    pub fn new(position: Point<f64>, direction: f64) -> Agent {
        Agent{
            speed: 0.0008,
            position,
            direction,
            ray_count: 90.0,
            fov: 0.4,
            visibility: 0.1,
            rays: vec![]
        }
    }

    pub fn cast_rays(&mut self) {
        self.rays.clear();
        for i in 0..self.ray_count as i32 {
            let x = i as f64 / self.ray_count - 0.5;
            let angle = x.atan2(self.fov);
            self.rays.push(Ray::new(angle, self.visibility, self.direction, self.position.clone()))
        }
    }

    pub fn step(&mut self, direction_change: f64) {
        self.direction += direction_change;
        self.position = Point::new(
            self.position.x() + self.speed * self.direction.cos(),
            self.position.y() + self.speed * self.direction.sin(),
        );
        self.cast_rays();
    }
}

impl Environment {
    pub fn new() -> Environment {
        let geojson_str = fs::read_to_string("polygons.json").unwrap();
        let geojson = geojson_str.parse::<GeoJson>().unwrap();
        let mut collection: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
        let mut lines: Vec<LineString<_>> = vec![];
        for i in collection.iter_mut() {
            match i {
                Geometry::Point(_) => {}
                Geometry::Line(_) => {}
                Geometry::LineString(ref x) => {
                    lines.push(x.clone())
                }
                Geometry::Polygon(ref x) => {
                    lines.push(x.exterior().clone())
                }
                Geometry::MultiPoint(_) => {}
                Geometry::MultiLineString(_) => {}
                Geometry::MultiPolygon(_) => {}
                Geometry::GeometryCollection(_) => {}
                Geometry::Rect(_) => {}
                Geometry::Triangle(_) => {}
            }
        }
        let mut xmin: f64 = 999.9;
        let mut ymin: f64 = 999.9;
        let mut xmax: f64 = 0.0;
        let mut ymax: f64 = 0.0;
        for line in lines.iter() {
            for point in line.points_iter() {
                xmin = xmin.min(point.x());
                ymin = ymin.min(point.y());
                xmax = xmax.max(point.x());
                ymax = ymax.max(point.y());
            }
        }
        let scalex = 1.0 * (xmax - xmin);
        let scaley = 1.0 * (ymax - ymin);
        for line in lines.iter_mut() {
            line.map_coords_inplace(|&(x, y)| ((x - xmin) / scalex, (y - ymin) / scaley));
        }
        Environment { lines }
    }

    fn intersections(linestring1: &LineString<f64>, linestring2: &LineString<f64>) -> Vec<Point<f64>> {
        let mut intersections = vec![];
        if linestring1.0.is_empty() || linestring2.0.is_empty() {
            return vec![];
        }
        for a in linestring1.lines() {
            for b in linestring2.lines() {
                let a_li = LineInterval::line_segment(a.clone());
                let b_li = LineInterval::line_segment(b.clone());
                match a_li.relate(&b_li) {
                    LineRelation::DivergentIntersecting(x) => intersections.push(x.clone()),
                    _ => {}
                }
            }
        }
        intersections
    }

    pub fn update_state(&self, agent: &mut Agent) {
        for ray in agent.rays.iter_mut() {
            for line in self.lines.iter() {
                let intersections = Environment::intersections(&ray.line_string, line);
                for intersection in intersections.iter() {
                    let length = intersection.euclidean_distance(&agent.position);
                    if length < ray.length {
                        ray.length = length;
                        ray.line_string = LineString(vec![ray.line_string.0[0], intersection.0])
                    }
                }
            }
        }
    }
}
use sdl2::rect;
use line_intersection::{LineInterval, LineRelation};

fn main() -> Result<(), String> {
    let scalex = 1000.0;
    let scaley = 1000.0;
    let min_distance_to_obstacle = 0.005;
    let mut last_distance_to_obstacle = 0.0;
    let mut _env = Environment::new();
    let mut agent = Agent::new(Point::new(0.45, 0.55), 2.0);
    agent.cast_rays();
    let mut direction_change = 0.0;


    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rust-sdl2 demo: Video", 1000, 1000)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 0, 255));

        let distance = agent.rays.iter().map(|x| x.length).fold(1000.0, |a, b| a.min(b));

        if distance < min_distance_to_obstacle {
            agent.speed = 0.0001;
            if distance > last_distance_to_obstacle {
                last_distance_to_obstacle = distance;
                direction_change = 0.0;
            } else {
                last_distance_to_obstacle = distance;
                direction_change += rand::random::<f64>() - 0.5;
            }
        } else {
            agent.speed = 0.0008;
            direction_change = 0.0;
        }
        agent.step(direction_change);

        for ray in &agent.rays {
            canvas.draw_line(rect::Point::new((ray.line_string.0[0].x * scalex) as i32, (ray.line_string.0[0].y * scaley) as i32), rect::Point::new((ray.line_string.0[1].x * scalex) as i32, (ray.line_string.0[1].y * scaley) as i32));
        }

        _env.update_state(&mut agent);

        canvas.set_draw_color(Color::RGB(255, 0, 0));

        for ray in &agent.rays {
            canvas.draw_line(rect::Point::new((ray.line_string.0[0].x * scalex) as i32, (ray.line_string.0[0].y * scaley) as i32), rect::Point::new((ray.line_string.0[1].x * scalex) as i32, (ray.line_string.0[1].y * scaley) as i32));
        }

        canvas.set_draw_color(Color::RGB(0, 255, 0));

        for line in &_env.lines {
            for line_segment in line.lines() {
                canvas.draw_line(
                    rect::Point::new(
                        (line_segment.start.x * scalex) as i32,
                        (line_segment.start.y * scaley) as i32,
                    ),
                    rect::Point::new(
                        (line_segment.end.x * scalex) as i32,
                        (line_segment.end.y * scaley) as i32,
                    )
                );
            }
        }

        canvas.present();
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