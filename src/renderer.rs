use crate::ray::Ray;
use core::option::Option::Some;
use geo::LineString;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use sdl2::{pixels, rect, EventPump};

pub struct Renderer {
    pub canvas: WindowCanvas,
    pub event_pump: EventPump,
    pub scalex: f64,
    pub scaley: f64,
}

impl Renderer {
    pub fn new(_scalex: f64, _scaley: f64) -> Renderer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo: Video", 1000, 1000)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump().unwrap();
        Renderer {
            canvas,
            event_pump,
            scalex: 1000.0,
            scaley: 1000.0,
        }
    }

    pub fn handle_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return true,
                _ => {}
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    pub fn render_rays<C: Into<pixels::Color>>(&mut self, rays: &Vec<Ray>, color: C) {
        self.canvas.set_draw_color(color);
        for ray in rays {
            self.canvas.draw_line(
                rect::Point::new(
                    (ray.line_string.0[0].x * self.scalex) as i32,
                    (ray.line_string.0[0].y * self.scaley) as i32,
                ),
                rect::Point::new(
                    (ray.line_string.0[1].x * self.scalex) as i32,
                    (ray.line_string.0[1].y * self.scaley) as i32,
                ),
            ).unwrap();
        }
    }

    pub fn render_lines<C: Into<pixels::Color>>(&mut self, lines: &Vec<LineString<f64>>, color: C) {
        self.canvas.set_draw_color(color);
        for line in lines {
            for line_segment in line.lines() {
                self.canvas.draw_line(
                    rect::Point::new(
                        (line_segment.start.x * self.scalex) as i32,
                        (line_segment.start.y * self.scaley) as i32,
                    ),
                    rect::Point::new(
                        (line_segment.end.x * self.scalex) as i32,
                        (line_segment.end.y * self.scaley) as i32,
                    ),
                ).unwrap();
            }
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}
