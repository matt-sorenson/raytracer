use super::ray_vs_scene;
use super::ray_vs_scene_shadow;
use super::scene::AntiAliasType;
use super::shapes::*;
use super::Intersection;
use super::Scene;

use log::info;

use std::time::{Duration, Instant};

use rand::distributions::OpenClosed01;
use rand::Rng;
use rand_distr::{Distribution, UnitDisc};

use float_cmp::approx_eq;

use super::Float3;

pub trait Canvas {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Float3);
    fn present(&mut self);
}

fn get_normal(normal: Float3) -> Float3 {
    if approx_eq!(f64, normal.dot(&normal), 1.0) {
        return normal;
    }

    normal.normalize()
}

const EPSILON: f64 = 0.0012;

fn local_illumination(
    ray: &Ray,
    scene: &Scene,
    intersection: &Intersection,
    material: &Material,
    specular: f64,
) -> Float3 {
    let normal = get_normal(intersection.normal);
    let position = intersection.t * ray.direction + ray.origin;

    let mut out = scene.ambient;

    let mut shadow_feeler = Ray {
        // Jump slightly up from the surface so it doesn't intersect itself.
        origin: position + (normal * EPSILON),
        direction: Float3::new(0.0, 0.0, 0.0),
    };

    let shadow_count = 1;

    for light in scene.lights.iter() {
        let light_direction = light.center - position;

        // Determine if the point of intersection is in shadow
        let mut shadow = 1.0;
        if shadow_count == 1 || approx_eq!(f64, light.radius, 0.0) {
            shadow_feeler.direction = light_direction;

            if ray_vs_scene_shadow(&shadow_feeler, scene) {
                shadow = 0.0;
            }
        } else {
            let mut shadow_counter = 0;
            for _ in 0..shadow_count {
                // Generate polar coordinates to then convert to euclidean coordinates
                // on the plane with it's point at the light and it's normal the vector
                // from the 'light' - 'position'
                let v: [f64; 2] = UnitDisc.sample(&mut rand::thread_rng());
                let v = [v[0] * light.radius, v[1] * light.radius];

                let i1 = light_direction
                    .cross(&Float3::new(0.0, 0.0, 1.0))
                    .normalize();
                let i2 = light_direction.cross(&i1).normalize();

                shadow_feeler.direction = light.center + (i1 * v[0]) + (i2 * v[1]) - position;

                if ray_vs_scene_shadow(&shadow_feeler, scene) {
                    shadow_counter += 1;
                }
            }

            shadow = (shadow_count - shadow_counter) as f64 / shadow_count as f64;
        }

        // Diffuse Light
        let light_direction = light_direction.normalize();
        let n_dot_l = f64::max(0.0, normal.dot(&light_direction));
        let diffuse_factor = shadow * n_dot_l;
        out.x += diffuse_factor * material.diffuse.x * light.color.x;
        out.y += diffuse_factor * material.diffuse.y * light.color.y;
        out.z += diffuse_factor * material.diffuse.z * light.color.z;

        // Specular Light
        let l = (2.0 * normal.dot(&light_direction) * normal) - light_direction;
        let v_dot_l = ray.direction.dot(&-l);
        if v_dot_l > 0.0 {
            out += v_dot_l.powf(material.specular_power) * specular * light.color;
        }
    }

    out
}

fn reflect(reflection_vector: &Float3, reflected: &Float3) -> Float3 {
    let r_dot_rv = 2.0 * reflected.dot(reflection_vector);

    (reflected - (r_dot_rv * reflection_vector)).normalize()
}

fn transmit(nit: f64, normal: &Float3, from: &Float3) -> Option<Float3> {
    let f_dot_n = from.dot(&normal);
    let cos_t = 1.0 - (nit * nit) * (1.0 - (f_dot_n * f_dot_n));

    if cos_t <= 0.0 {
        return None;
    }

    let inv = if f_dot_n < 0.0 { 1.0 } else { -1.0 };
    let transmission = cos_t.sqrt() * inv;

    Some((((transmission + (nit * f_dot_n)) * normal) - (nit * from)).normalize())
}

/// https://en.wikipedia.org/wiki/Fresnel_equations#Power_(intensity)_reflection_and_transmission_coefficients
/// Arguments:
/// `n_i`: The index of refraction of the transmission medium of the ray.
/// `n_t`: The index of refraction of the object material to transmit into.
/// `u_i`: The magnetic permeability of the transmission medium
/// `u_t`: The magnetic permeability of the object material to transmit into.
/// `cos_theta_i`: cos(θ_i) where θ_i is the angle of incidence
fn fresnel(n_i: f64, n_t: f64, u_i: f64, u_t: f64, cos_theta_i: f64) -> f64 {
    let nit = n_i / n_t;
    let uit = u_i / u_t;

    let determinate = 1.0 - ((nit * nit) * (1.0 - (cos_theta_i * cos_theta_i)));

    if determinate < 0.0 {
        return 1.0;
    }

    // θ_t Angle of transmission
    let cos_theta_t = determinate.sqrt();

    let e_r_perp = (nit * cos_theta_i) - (uit * cos_theta_t);
    let e_i_perp = (nit * cos_theta_i) + (uit * cos_theta_t);

    let e_r_par = (uit * cos_theta_i) - (nit * cos_theta_t);
    let e_i_par = (uit * cos_theta_i) + (nit * cos_theta_t);

    let e_perp = e_r_perp / e_i_perp;
    let e_par = e_r_par / e_i_par;

    0.5 * ((e_perp * e_perp) + (e_par * e_par))
}

fn cast_ray(ray: &Ray, scene: &Scene, depth: u32, n_i: f64) -> Float3 {
    let mut color = Float3::new(0.0, 0.0, 0.0);

    if depth == 0 {
        return color;
    }

    let res = ray_vs_scene(&ray, &scene);

    if res.is_none() {
        return color;
    }

    let (intersection, material) = res.unwrap();

    let mut n_t = 1.0;
    let mut u_i = 1.0; // μ_i in fresnel equation
    let mut u_t = 1.0; // μ_t in fresnel equation
    let mut attenuation = scene.air_attenuation;

    if approx_eq!(f64, n_i, 1.0) {
        n_t = material.index_of_refraction;
        u_t = material.magnetic_permeability;
    } else {
        u_i = material.magnetic_permeability;
        attenuation = material.attenuation;
    }

    let r_dot_n = ray.direction.dot(&intersection.normal).abs();
    let r_ = fresnel(n_i, n_t, u_i, u_t, r_dot_n);
    let transmission_coefficient = material.specular_coefficient * (1.0 - r_);
    let reflection_coefficient = material.specular_coefficient * r_;

    let normal = intersection.normal;

    if approx_eq!(f64, n_i, 1.0) {
        color += local_illumination(ray, scene, &intersection, &material, reflection_coefficient);
    }

    if depth > 1 {
        if !approx_eq!(f64, reflection_coefficient, 0.0) {
            let normal_fudge_factor = if n_i == 1.0 { EPSILON } else { -EPSILON };
            let point = ray.origin + (ray.direction * intersection.t) + (normal * normal_fudge_factor);

            let reflection = Ray {
                origin: point,
                direction: reflect(&normal, &ray.direction),
            };
            color += reflection_coefficient * cast_ray(&reflection, scene, depth - 1, n_i);
        }

        if !approx_eq!(f64, transmission_coefficient, 0.0) {
            if let Some(direction) = transmit(n_i / n_t, &normal, &-ray.direction) {
                let normal_fudge_factor = if n_i == 1.0 { -EPSILON } else { EPSILON };
                let point = ray.origin + (ray.direction * intersection.t) + (normal * normal_fudge_factor);

                let transmission = Ray {
                    origin: point,
                    direction,
                };

                color += transmission_coefficient * cast_ray(&transmission, scene, depth - 1, n_t);
            }
        }
    }

    color.x = attenuation.x.powf(intersection.t) * color.x;
    color.y = attenuation.y.powf(intersection.t) * color.y;
    color.z = attenuation.z.powf(intersection.t) * color.z;

    color
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    (a * (1.0 - t)) + (b * t)
}

fn calculate_rays(scene: &Scene, x: u32, y: u32) -> Vec<Ray> {
    let mut rays = Vec::new();

    let dx = 2.0 / (scene.width as f64);
    let dy = 2.0 / (scene.height as f64);

    let min_x = -1.0 + ((x as f64) - 0.5) * dx;
    let min_y = -1.0 + ((y as f64) - 0.5) * dy;
    let max_x = -1.0 + ((x as f64) + 0.5) * dx;
    let max_y = -1.0 + ((y as f64) + 0.5) * dy;

    fn create_ray(scene: &Scene, x: f64, y: f64) -> Ray {
        let viewport_position =
            scene.viewport_origin + (x * scene.viewport_x_axis) + (y * scene.viewport_y_axis);

        Ray {
            origin: scene.eye_position,
            direction: (viewport_position - scene.eye_position).normalize(),
        }
    }

    let aa_type = if 1 == scene.aa_rate {
        AntiAliasType::None
    } else {
        scene.aa_type
    };

    match aa_type {
        AntiAliasType::None => {
            let x = -1.0 + (x as f64) * dx;
            let y = -1.0 + (y as f64) * dy;
    
            rays.push(create_ray(&scene, x, y));
        },
        AntiAliasType::SuperSample => {
            for i in 0..(scene.aa_rate) {
                for j in 0..(scene.aa_rate) {
                    let x = lerp(min_x, max_x, (i as f64) / (scene.aa_rate as f64));
                    let y = lerp(min_y, max_y, (j as f64) / (scene.aa_rate as f64));
    
                    rays.push(create_ray(&scene, x, y));
                }
            }
        },
        AntiAliasType::MonteCarlo => {
            for _ in 0..(scene.aa_rate) {
                for _ in 0..(scene.aa_rate) {
                    let x = lerp(min_x, max_x, rand::thread_rng().sample(OpenClosed01));
                    let y = lerp(min_y, max_y, rand::thread_rng().sample(OpenClosed01));
    
                    rays.push(create_ray(&scene, x, y));
                }
            }
        }
    }

    rays
}

fn calculate_pixel_color(scene: &Scene, x: u32, y: u32, max_depth: u32) -> Float3 {
    let rays = calculate_rays(&scene, x, y);

    let mut color = Float3::new(0.0, 0.0, 0.0);
    for ray in rays.iter() {
        color += cast_ray(ray, scene, max_depth, 1.0);
    }

    color /= rays.len() as f64;

    color
}

pub fn render_scene<T: Canvas>(scene: &Scene, canvas: &mut T, start_y: u32, max_depth: u32) -> u32 {
    let start_time = Instant::now();

    for y in start_y..scene.height {
        for x in 0..scene.width {
            canvas.set_pixel(x, y, &calculate_pixel_color(scene, x, y, max_depth));
        }

        if y % (scene.width / 10) == 0 {
            info!("{}%", ((y as f64) / (scene.width as f64)) * 100.0);
        }

        canvas.present();

        if y != scene.height && start_time.elapsed() > Duration::from_millis(16) {
            return y + 1;
        }
    }

    info!("DONE!");

    u32::MAX
}
