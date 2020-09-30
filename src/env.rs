use crate::agent::Agent;
use geo::euclidean_distance::EuclideanDistance;
use geo::map_coords::MapCoordsInplace;
use geo::{Geometry, GeometryCollection, LineString, Point, Polygon};
use geojson::{quick_collection, GeoJson};
use line_intersection::{LineInterval, LineRelation};
use rayon::prelude::*;
use std::fs;
use crate::ray::Ray;
use geo::intersects::Intersects;

pub struct Env {
    pub line_strings: Vec<LineString<f64>>,
    pub scalex: f64,
    pub scaley: f64,
    /*
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
     */
}

impl Env {
    pub fn load_json() -> GeometryCollection<f64> {
        let geojson_str = fs::read_to_string("polygons.json").unwrap();
        let geojson = geojson_str.parse::<GeoJson>().unwrap();
        quick_collection(&geojson).unwrap()
    }

    pub fn collection_as_line_strings(
        mut collection: GeometryCollection<f64>,
    ) -> Vec<LineString<f64>> {
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

    pub fn scale_line_strings(
        scalex: f64,
        scaley: f64,
        xmin: f64,
        ymin: f64,
        lines: &mut Vec<LineString<f64>>,
    ) {
        for line in lines.iter_mut() {
            line.map_coords_inplace(|&(x, y)| ((x - xmin) / scalex, (y - ymin) / scaley));
        }
    }

    pub fn get_line_strings() -> (Vec<LineString<f64>>, f64, f64) {
        let collection = Env::load_json();
        let mut lines = Env::collection_as_line_strings(collection);
        let (scalex, scaley, xmin, ymin, _xmax, _ymax) = Env::calculate_scales(&lines);
        Env::scale_line_strings(scalex, scaley, xmin, ymin, &mut lines);
        (lines, scalex, scaley)

    }

    pub fn new() -> Env {
        let (line_strings, scalex, scaley) = Env::get_line_strings();
        Env {
            line_strings,
            scalex,
            scaley,
            /*
            xmin,
            ymin,
            xmax,
            ymax,
            */
        }
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
        let intersecting_line_strings = Env::find_culled(&mut agent.rays, &self.line_strings, agent.position);
        Env::find_intersections_seq(&mut agent.rays, &intersecting_line_strings, agent.position)
    }

    pub fn find_intersections(ray: &mut Ray, line_strings: &Vec<&LineString<f64>>, origin_position: Point<f64>) {
        for line in line_strings.iter() {
            let intersections = Env::intersections(&ray.line_string, line);
            for intersection in intersections.iter() {
                let length = intersection.euclidean_distance(&origin_position);
                if length < ray.length {
                    ray.length = length;
                    ray.line_string = LineString(vec![ray.line_string.0[0], intersection.0])
                }
            }
        }
    }

    pub fn find_culled<'a>(rays: &Vec<Ray>, line_strings: &'a Vec<LineString<f64>>, origin_position: Point<f64>) -> Vec<&'a LineString<f64>> {
        let polygon = Polygon::new(LineString::from(vec![
            origin_position.x_y(), // origin
            rays[0].line_string.0[1].x_y(),
            rays[(rays.len() as f64 / 2.0).floor() as usize].line_string.0[1].x_y(),
            rays[rays.len()-1].line_string.0[1].x_y(),
            origin_position.x_y(), // origin
        ]), vec![]);

        let mut intersecting_line_strings = vec![];
        for line_string in line_strings.iter() {
            if polygon.intersects(line_string) {
                intersecting_line_strings.push(line_string)
            }
        }
        intersecting_line_strings
    }

    pub fn find_intersections_seq(rays: &mut Vec<Ray>, line_strings: &Vec<&LineString<f64>>, origin_position: Point<f64>) {
        rays.iter_mut().for_each(|ray| {
            Env::find_intersections(ray, line_strings, origin_position)
        });

    }

    pub fn find_intersections_par(rays: &mut Vec<Ray>, line_strings: &Vec<&LineString<f64>>, origin_position: Point<f64>) {
        rays.par_iter_mut().for_each(|ray| {
            Env::find_intersections(ray, line_strings, origin_position)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::ray::Ray;
    use geo::{Point, Polygon, LineString, Rect};
    use crate::env::Env;
    use test::Bencher;
    use geo::prelude::Intersects;


    #[test]
    fn test_intersections() {
        let position = Point::new(0.5, 0.5);
        let mut rays = Ray::generate_rays(180.0, 0.4, 0.3, 0.0, position);
        let (line_strings, scalex, scaley) = Env::get_line_strings();

        let intersecting_line_strings = Env::find_culled(&mut rays, &line_strings, position);
        Env::find_intersections_seq(&mut rays, &intersecting_line_strings, position)
    }

    #[bench]
    fn test_calc_intersections_par(b: &mut Bencher) {
        let position = Point::new(0.5, 0.5);
        let mut rays = Ray::generate_rays(180.0, 0.4, 0.3, 0.0, position);
        let (line_strings, scalex, scaley) = Env::get_line_strings();

        b.iter(|| Env::find_intersections_par(&mut rays, &line_strings.iter().map(|ls|ls).collect(), position));
    }

    #[bench]
    fn test_calc_intersections_seq(b: &mut Bencher) {
        let position = Point::new(0.5, 0.5);
        let mut rays = Ray::generate_rays(180.0, 0.4, 0.3, 0.0, position);
        let (line_strings, scalex, scaley) = Env::get_line_strings();
        b.iter(|| Env::find_intersections_seq(&mut rays, &line_strings.iter().map(|ls|ls).collect(), position));
    }

    #[bench]
    fn test_calc_intersections_par_with_culling(b: &mut Bencher) {
        let position = Point::new(0.5, 0.5);
        let mut rays = Ray::generate_rays(180.0, 0.4, 0.3, 0.0, position);
        let (line_strings, scalex, scaley) = Env::get_line_strings();
        b.iter(|| {
            let intersecting_line_strings = Env::find_culled(&mut rays, &line_strings, position);
            Env::find_intersections_par(&mut rays, &intersecting_line_strings, position)
        });
    }

    #[bench]
    fn test_calc_intersections_seq_with_culling(b: &mut Bencher) {
        let position = Point::new(0.5, 0.5);
        let mut rays = Ray::generate_rays(180.0, 0.4, 0.3, 0.0, position);
        let (line_strings, scalex, scaley) = Env::get_line_strings();
        b.iter(|| {
            let intersecting_line_strings = Env::find_culled(&mut rays, &line_strings, position);
            Env::find_intersections_seq(&mut rays, &intersecting_line_strings, position)
        });
    }

}
