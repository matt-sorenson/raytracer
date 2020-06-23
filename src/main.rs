use std::env;
use std::thread::sleep;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Point;


mod raytracer;
use raytracer::render::Canvas;
use raytracer::*;

fn float3_to_color(input: &Float3) -> Color {
    Color::RGB(
        f64::max(0.0, f64::min(255.0, input.x * 255.0)) as u8,
        f64::max(0.0, f64::min(255.0, input.y * 255.0)) as u8,
        f64::max(0.0, f64::min(255.0, input.z * 255.0)) as u8,
    )
}

struct Window {
    context: sdl2::Sdl,
    canvas: sdl2::render::WindowCanvas,
    // width: u32,
    height: u32,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Window {
        let context = sdl2::init().unwrap();
        let window = context
            .video()
            .unwrap()
            .window("raytracer", width, height)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Window { context, canvas, /*width,*/ height }
    }

    pub fn event_pump(&self) -> bool {
        let mut event_pump = self.context.event_pump().unwrap();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return false,
                _ => {}
            }
        }

        true
    }
}

impl Canvas for Window {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Float3) {
        let sdl_color = float3_to_color(&color);
        self.canvas.set_draw_color(sdl_color);

        // (0,0) on the window is the top left corner.
        // (0,0) on the renderer is the bottom left.
        let y = self.height - y;

        self.canvas
            .draw_point(Point::new(x as i32, y as i32))
            .expect("Failed to draw point");
    }

    fn present(&mut self) {
        self.canvas.present();
    }
}

fn main() {
    simple_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("You must provide a filename");
    }

    let scene = Scene::from_file(&args[1]);

    let mut window = Window::new(scene.width, scene.height);

    let mut y = 0;

    'running: loop {
        if !window.event_pump() {
            break 'running;
        }

        if y != u32::MAX {
            y = raytracer::render::render_scene(&scene, &mut window, y, 10);
        }

        window.present();
        sleep(Duration::from_millis(1));
    }
}
