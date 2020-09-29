use std::fs;
use line_intersection::{LineInterval, LineRelation};
use geo::{LineString, GeometryCollection, Geometry, Point};
use geojson::{GeoJson, quick_collection};
use geo::map_coords::MapCoordsInplace;
use crate::agent::Agent;
use geo::euclidean_distance::EuclideanDistance;
use rayon::prelude::*;

pub struct Environment {
    pub lines: Vec<LineString<f64>>
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
        let agent_position = agent.position.clone();
        agent.rays.par_iter_mut().for_each(|ray| {
            for line in self.lines.iter() {
                let intersections = Environment::intersections(&ray.line_string, line);
                for intersection in intersections.iter() {
                    let length = intersection.euclidean_distance(&agent_position);
                    if length < ray.length {
                        ray.length = length;
                        ray.line_string = LineString(vec![ray.line_string.0[0], intersection.0])
                    }
                }
            }
        });
    }
}
