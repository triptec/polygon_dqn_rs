use crate::agent::Agent;
use geo::euclidean_distance::EuclideanDistance;
use geo::map_coords::MapCoordsInplace;
use geo::{Geometry, GeometryCollection, LineString, Point};
use geojson::{quick_collection, GeoJson};
use line_intersection::{LineInterval, LineRelation};
use rayon::prelude::*;
use std::fs;

pub struct Env {
    pub lines: Vec<LineString<f64>>,
    pub scalex: f64,
    pub scaley: f64,
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl Env {
    pub fn load_json() -> GeometryCollection<f64> {
        let geojson_str = fs::read_to_string("polygons.json").unwrap();
        let geojson = geojson_str.parse::<GeoJson>().unwrap();
        quick_collection(&geojson).unwrap()
    }

    pub fn collection_as_line_strings(mut collection: GeometryCollection<f64>) -> Vec<LineString<f64>> {
        let mut lines: Vec<LineString<_>> = vec![];
        for i in collection.iter_mut() {
            match i {
                Geometry::Point(_) => {}
                Geometry::Line(_) => {}
                Geometry::LineString(ref x) => lines.push(x.clone()),
                Geometry::Polygon(ref x) => lines.push(x.exterior().clone()),
                Geometry::MultiPoint(_) => {}
                Geometry::MultiLineString(_) => {}
                Geometry::MultiPolygon(_) => {}
                Geometry::GeometryCollection(_) => {}
                Geometry::Rect(_) => {}
                Geometry::Triangle(_) => {}
            }
        }
        lines
    }

    pub fn calculate_scales(lines: &Vec<LineString<f64>>) -> (f64, f64, f64, f64, f64, f64) {
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
        (scalex, scaley, xmin, ymin, xmax, ymax)
    }

    pub fn scale_line_strings(scalex: f64, scaley: f64, xmin: f64, ymin: f64, lines: &mut Vec<LineString<f64>>) {
        for line in lines.iter_mut() {
            line.map_coords_inplace(|&(x, y)| ((x - xmin) / scalex, (y - ymin) / scaley));
        }
    }

    pub fn new() -> Env {
        let collection = Env::load_json();
        let mut lines = Env::collection_as_line_strings(collection);
        let (scalex, scaley, xmin, ymin, xmax, ymax) = Env::calculate_scales(&lines);

        Env::scale_line_strings(scalex, scaley, xmin, ymin, &mut lines);


        Env { lines, scalex, scaley, xmin, ymin, xmax, ymax }
    }

    fn intersections(
        linestring1: &LineString<f64>,
        linestring2: &LineString<f64>,
    ) -> Vec<Point<f64>> {
        let mut intersections = vec![];
        if linestring1.0.is_empty() || linestring2.0.is_empty() {
            return vec![];
        }
        for a in linestring1.lines() {
            for b in linestring2.lines() {
                let a_li = LineInterval::line_segment(a);
                let b_li = LineInterval::line_segment(b);
                match a_li.relate(&b_li) {
                    LineRelation::DivergentIntersecting(x) => intersections.push(x),
                    _ => {}
                }
            }
        }
        intersections
    }

    pub fn update_state(&self, agent: &mut Agent) {
        let agent_position = agent.position;
        agent.rays.par_iter_mut().for_each(|ray| {
            for line in self.lines.iter() {
                let intersections = Env::intersections(&ray.line_string, line);
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}