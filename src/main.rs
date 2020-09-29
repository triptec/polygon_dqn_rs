use geo::{LineString, GeometryCollection, Geometry, Point, Coordinate, Line};
use std::{fs};
use geojson::{GeoJson, quick_collection};
use rand::prelude::*;

use geo::algorithm::map_coords::MapCoordsInplace;
use geo::intersects::Intersects;
use num_traits::Float;
use geo::algorithm::euclidean_distance::EuclideanDistance;
pub struct Environment {
    pub lines: Vec<LineString<f64>>
}

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
            speed: 0.01,
            position,
            direction,
            ray_count: 30.0,
            fov: 4.0,
            visibility: 0.5,
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
        self.cast_rays();
        self.position = Point::new(
            self.position.x() + self.speed * self.direction.cos(),
            self.position.y() + self.speed * self.direction.sin(),
        )
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
        /*

        dbg!(xmin, ymin, xmax, ymax, scalex, scaley);
        xmin = 999.9;
        ymin = 999.9;
        xmax = 0.0;
        ymax = 0.0;
        for line in lines.iter() {
            for point in line.points_iter() {
                xmin = xmin.min(point.x());
                ymin = ymin.min(point.y());
                xmax = xmax.max(point.x());
                ymax = ymax.max(point.y());
            }
        }
        dbg!(xmin, ymin, xmax, ymax, scalex, scaley);

        dbg!(&lines);
        
         */
        Environment { lines }
    }
    fn intersect(line1: &Line<f64>, line2: &Line<f64>) -> Option<Point<f64>> {
        let a1 = line1.end.y - line1.start.y;
        let b1 = line1.start.x - line1.end.x;
        let c1 = a1 * line1.start.x + b1 * line1.start.y;

        let a2 = line2.end.y - line2.start.y;
        let b2 = line2.start.x - line2.end.x;
        let c2 = a2 * line2.start.x + b2 * line2.start.y;

        let delta = a1 * b2 - a2 * b1;

        if delta == 0.0 {
            return None;
        }

        Some(Point::new(
            (b2 * c1 - b1 * c2) / delta,
            (a1 * c2 - a2 * c1) / delta
        ))
    }

    fn intersections(linestring1: &LineString<f64>, linestring2: &LineString<f64>) -> Vec<Point<f64>> {
        let mut intersections = vec![];
        if linestring1.0.is_empty() || linestring2.0.is_empty() {
            return vec![];
        }
        for a in linestring1.lines() {
            for b in linestring2.lines() {
                match Environment::intersect(&a, &b) {
                    Some(x) => intersections.push(x),
                    None => {}
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

fn main() {
    let mut _env = Environment::new();
    let mut agent = Agent::new(Point::new(0.5, 0.5), 2.0);
    agent.cast_rays();
    let mut direction_change = 0.0;
    loop {
        let distance = agent.rays.iter().map(|x| x.length).fold(1000.0, |a, b| a.min(b));
        if distance < 0.1 {
            direction_change += rand::random::<f64>() - 0.5;
        } else {
            direction_change = 0.0;
        }
        agent.step(direction_change);
        _env.update_state(&mut agent);
    }
    println!("Hello, world!");
}
