extern crate nalgebra as na;
use std::vec::Vec;

use std::option::Option;

pub type Float3 = na::Vector3<f64>;

pub mod render;
pub mod scene;
pub mod shapes;
pub use crate::scene::Scene;
use crate::shapes::*;

#[derive(Debug, Copy, Clone)]
pub struct Intersection {
    pub t: f64,
    pub normal: Float3,
}

pub fn ray_vs_scene_helper(ray: &Ray, scene: &Scene, break_on_hit: bool, max_t: f64) -> Option<(Intersection, Material)> {
    let mut t = max_t;
    let mut out: Option<(Intersection, Material)> = None;

    for shape in scene.spheres.iter() {
        if let Some(res) = ray_vs_sphere(&ray, &shape, t) {
            t = res.t;
            out = Some((res, shape.material));

            if break_on_hit {
                return out;
            }
        }
    }

    for shape in scene.ellipsoids.iter() {
        if let Some(res) = ray_vs_ellipsoid(&ray, &shape, t) {
            t = res.t;
            out = Some((res, shape.material));

            if break_on_hit {
                return out;
            }
        }
    }

    for shape in scene.rhombohedrons.iter() {
        if let Some(res) = ray_vs_rhombohedron(&ray, &shape, t) {
            t = res.t;
            out = Some((res, shape.material));

            if break_on_hit {
                return out;
            }
        }
    }

    for shape in scene.polygons.iter() {
        if let Some(res) = ray_vs_polygon(&ray, &shape, t) {
            t = res.t;
            out = Some((res, shape.material));

            if break_on_hit {
                return out;
            }
        }
    }

    out
}

pub fn ray_vs_scene_shadow(ray: &Ray, scene: &Scene) -> bool {
    ray_vs_scene_helper(ray, scene, true, 1.0).is_some()
}

pub fn ray_vs_scene(ray: &Ray, scene: &Scene) -> Option<(Intersection, Material)> {
    ray_vs_scene_helper(ray, scene, false, f64::MAX)
}

// If the ray would also exit the sphere provide that intersection too.
fn ray_vs_sphere2(ray: &Ray, sphere: &Sphere) -> (u32, Vec<Intersection>) {
    let pc = ray.origin - sphere.center;

    let a = ray.direction.dot(&ray.direction);
    let b = 2.0 * pc.dot(&ray.direction);
    let c = pc.dot(&pc) - (sphere.radius * sphere.radius);

    let discriminant = (b * b) - (4.0 * a * c);

    if discriminant < 0.0 {
        return (0, Vec::new());
    }

    let discriminant = discriminant.sqrt();
    
    let t1 = (-b - discriminant) / (2.0 * a);
    let t2 = (-b + discriminant) / (2.0 * a);

    let n1 = (ray.origin + (t1 * ray.direction)) - sphere.center;
    let n2 = (ray.origin + (t2 * ray.direction)) - sphere.center;

    let count = if t2 < 0.0 {
        0u32
    } else if t1 < 0.0 {
        1u32
    } else {
        2u32
    };

    (count, vec!(Intersection {t: t1, normal: n1}, Intersection {t: t2, normal: n2}))
}

fn ray_vs_sphere(ray: &Ray, sphere: &Sphere, max_t: f64) -> Option<Intersection> {
    let (count, result) = ray_vs_sphere2(&ray, &sphere);

    if 2 == count {
        if result[0].t < max_t {
            return Some(result[0]);
        }
    } else if 1 == count {
        if result[1].t < max_t {
            return Some(result[1]);
        }
    }

    None
}

fn ray_vs_rhombohedron(ray: &Ray, rhombohedron: &Rhombohedron, max_t: f64) -> Option<Intersection> {
    let mut t: [f64; 2] = [0.0, max_t];
    let mut normals: [Float3; 2] = [Float3::new(0.0, 0.0, 0.0), Float3::new(0.0, 0.0, 0.0)];

    for plane in rhombohedron.planes.iter() {
        let d_dot_n = ray.direction.dot(&plane.normal);
        let op_dot_n = (ray.origin - plane.point).dot(&plane.normal);

        if d_dot_n < 0.0 {
            let t_int = -op_dot_n / d_dot_n;
            if t_int > t[0] {
                t[0] = t_int;
                normals[0] = plane.normal;
            }
        } else if d_dot_n > 0.0 {
            let t_int = -op_dot_n / d_dot_n;
            if t_int < t[1] {
                t[1] = t_int;
                normals[1] = plane.normal
            }
        } else if op_dot_n > 0.0 {
            // In this case the ray is parrallel to the plane & outside the
            // half-space containing the rhombohedron.
            return None;
        }
    }

    if t[0] > t[1] {
        return None;
    }

    let (t, normal) = if 0.0 == t[0] {
        (t[1], normals[1])
    } else {
        (t[0], normals[0])
    };

    if t > max_t {
        return None;
    }

    Some(Intersection{ t, normal })
}

fn ray_vs_plane(ray: &Ray, plane: &Plane, max_t: f64) -> Option<Intersection> {
    let d_dot_n = ray.direction.dot(&plane.normal);

    if 0.0 == d_dot_n {
        return None;
    }

    let t = -(ray.origin.dot(&plane.normal) - plane.normal.dot(&plane.point)) / d_dot_n;

    if t < 0.0 || t > max_t {
        return None;
    }

    Some(Intersection {
        t,
        normal: plane.normal,
    })
}

fn ray_vs_polygon(ray: &Ray, polygon: &Polygon, max_t: f64) -> Option<Intersection> {
    let result = ray_vs_plane(&ray, &polygon.plane, max_t);
    if result.is_none() {
        return None;
    }

    let intersection = result.unwrap();

    let scaled_direction = Float3::new(
        ray.direction.x * intersection.t,
        ray.direction.y * intersection.t,
        ray.direction.z * intersection.t,
    );

    let point = ray.origin + scaled_direction;

    for triangle in polygon.triangles.iter() {
        if triangle.contains(&point) {
            return Some(intersection);
        }
    }

    None
}

fn ray_vs_ellipsoid(ray: &Ray, ellipsoid: &Ellipsoid, max_t: f64) -> Option<Intersection> {
    // Transform the ray into a space where the ellipsoid is a sphere of radius 1
    // centered at the origin.
    let e_space_ray = Ray {
        origin: ellipsoid.inverse * (ray.origin - ellipsoid.center),
        direction: ellipsoid.inverse * ray.direction
    };

    let e_space_sphere = Sphere {
        center: Float3::new(0.0, 0.0, 0.0),
        radius: 1.0,
        material: ellipsoid.material
    };

    if let Some(intersection) = ray_vs_sphere(&e_space_ray, &e_space_sphere, max_t) {
        return Some(Intersection {
            t: intersection.t,
            normal: (ellipsoid.inverse_transpose * intersection.normal).normalize(),
        });
    }

    None
}
