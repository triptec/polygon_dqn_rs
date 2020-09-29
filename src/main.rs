use geo::{LineString, GeometryCollection, Geometry, Coordinate, Point};
use std::{fs};
use geojson::{GeoJson, quick_collection};




use geo::algorithm::map_coords::MapCoordsInplace;
use geo::intersects::Intersects;
use num_traits::Float;

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
        Environment{lines}
    }

    fn intersects<T: Float>(linestring1: &LineString<T>, linestring2: &LineString<T>) -> bool {
        if linestring1.0.is_empty() || linestring2.0.is_empty() {
            return false;
        }
        for a in linestring1.lines() {
            for b in linestring2.lines() {
                let u_b = b.dy() * a.dx() - b.dx() * a.dy();
                if u_b == T::zero() {
                    continue;
                }
                let ua_t = b.dx() * (a.start.y - b.start.y) - b.dy() * (a.start.x - b.start.x);
                let ub_t = a.dx() * (a.start.y - b.start.y) - a.dy() * (a.start.x - b.start.x);
                let u_a = ua_t / u_b;
                let u_b = ub_t / u_b;
                if (T::zero() <= u_a)
                    && (u_a <= T::one())
                    && (T::zero() <= u_b)
                    && (u_b <= T::one())
                {
                    return true;
                    //return Some(Point(Coordinate{x: a1_x + u_a * (a2_x - a1_x), y:a1_y + u_a * (a2_y - a1_y)}));
                }
            }
        }
        false
    }

    pub fn get_state(&self, rays: Vec<LineString<f64>>) {
        for ray in rays.iter() {
            for line in self.lines.iter() {
                line.intersects(ray);
            }
        }
    }
}

fn main() {
    let _env = Environment::new();
    println!("Hello, world!");
}
