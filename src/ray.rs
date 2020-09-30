use geo::{Coordinate, LineString, Point};

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
                Coordinate {
                    x: player_position.x(),
                    y: player_position.y(),
                },
                Coordinate {
                    x: player_position.x() + length * (player_angle + angle).cos(),
                    y: player_position.y() + length * (player_angle + angle).sin(),
                },
            ]),
        }
    }
}
