use geo::{LineString, GeometryCollection, Geometry};
use std::{fs};
use geojson::{GeoJson, quick_collection};




use geo::algorithm::map_coords::MapCoordsInplace;

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

}

fn main() {
    let _env = Environment::new();
    println!("Hello, world!");
}
