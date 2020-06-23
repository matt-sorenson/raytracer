use log::info;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::thread::sleep;
use std::time::Duration;

mod raytracer;
use raytracer::shapes::*;
use raytracer::scene::AntiAliasType;
use raytracer::render::Canvas;
use raytracer::*;

fn create_scene() -> Scene {
    let mut scene = Scene::new();

    scene.spheres.push(Sphere {
        center: Float3::new(0.5, 0.25, -0.5),
        radius: 0.25,

        material: Material {
            diffuse: Float3::new(0.5, 0.7, 0.5),
            specular_coefficient: 0.3,
            specular_power: 70.0,
            attenuation: Float3::new(0.0, 0.0, 0.0),
            electric_permittivity: 1_000_000.0,
            magnetic_permeability: 1.0,
            index_of_refraction: 1000.0, // sqrt(electric_permittivity * magnetic_permiability)
        },
    });

    scene.rhombohedrons.push(Rhombohedron::from_corner_and_edges(
            Float3::new(-0.2623, 0.001, -0.7042),
            Float3::new(0.6495, 0.0, -0.375),
            Float3::new(-0.125, 0.0, -0.2165),
            Float3::new(0.0, 0.75, 0.0),
            Material {
                diffuse: Float3::new(0.3, 0.3, 0.5),
                specular_coefficient: 0.8,
                specular_power: 20.0,
                attenuation: Float3::new(0.5, 0.5, 0.5),
                electric_permittivity: 2.3716,
                magnetic_permeability: 1.0,
                index_of_refraction: f64::sqrt(2.3716 * 1.0),
            },
        ));

    scene.polygons.push(Polygon::from_vertices(
        vec![
            Float3::new(1.0, 0.0, 0.0),
            Float3::new(1.0, 0.0, -2.0),
            Float3::new(-1.0, 0.0, -2.0),
            Float3::new(-1.0, 0.0, 0.0),
        ],
        Material {
            diffuse: Float3::new(0.6, 0.6, 0.6),
            specular_coefficient: 0.4,
            specular_power: 20.0,
            attenuation: Float3::new(0.0, 0.0, 0.0),
            electric_permittivity: 1_000_000.0,
            magnetic_permeability: 1.0,
            index_of_refraction: 1000.0,
        },
    ));

    scene.ellipsoids.push(Ellipsoid::new(
        Float3::new(-0.5,0.5,-1.5),
        [Float3::new(0.25,0.0,0.0), Float3::new(0.0,0.5,0.0), Float3::new(0.0,0.0,0.25)],
        Material {
            diffuse: Float3::new(0.7, 0.5, 0.5),
            specular_coefficient: 0.3,
            specular_power: 70.0,
            attenuation: Float3::new(0.0, 0.0, 0.0),
            electric_permittivity: 1_000_000.0,
            magnetic_permeability: 1.0,
            index_of_refraction: 1000.0, // sqrt(electric_permittivity * magnetic_permiability)
        }));

    scene.lights.push(Light {
        center: Float3::new(-1.0, 1.0, 0.0),
        radius: 0.1,
        color: Float3::new(1.0, 1.0, 1.0),
    });

    scene.lights.push(Light {
        center: Float3::new(0.75, 0.5, 0.0),
        radius: 0.2,
        color: Float3::new(0.8, 0.8, 0.8),
    });

    scene.viewport_origin = Float3::new(0.0267612, 0.846193, -0.14023);
    scene.viewport_x_axis = Float3::new(0.343626, -0.274153, 0.238247);
    scene.viewport_y_axis = Float3::new(0.362222, 0.234501, -0.252595);
    scene.eye_position = scene.viewport_origin + Float3::new(0.0535224, 0.692386, 0.719539);

    scene.aa_type = AntiAliasType::SuperSample;
    scene.aa_rate = 1;

    let x_axis = scene.viewport_x_axis;
    let y_axis = scene.viewport_y_axis;

    scene.width = 860;
    scene.height = ((scene.width as f64) * y_axis.dot(&y_axis).sqrt() / x_axis.dot(&x_axis).sqrt()) as u32;

    info!("{}x{}", scene.width, scene.height);

    scene
}

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

    let scene = create_scene();
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
