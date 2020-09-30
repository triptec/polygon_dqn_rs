use crate::ray::Ray;
use geo::Point;

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
        Agent {
            speed: 0.0004,
            position,
            direction,
            ray_count: 20.0,
            fov: 0.4,
            visibility: 0.1,
            rays: vec![],
        }
    }

    pub fn cast_rays(&mut self) {
        self.rays.clear();
        self.rays = Ray::generate_rays(self.ray_count, self.fov, self.visibility, self.direction, self.position)
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
