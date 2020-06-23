use super::shapes::*;

use super::Float3;

#[derive(Debug, Copy, Clone)]
pub enum AntiAliasType {
    None,
    SuperSample,
    MonteCarlo,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub rhombohedrons: Vec<Rhombohedron>,
    pub polygons: Vec<Polygon>,
    pub ellipsoids: Vec<Ellipsoid>,
    pub lights: Vec<Light>,

    pub ambient: Float3,
    pub air_attenuation: Float3,

    pub viewport_origin: Float3,
    pub viewport_x_axis: Float3,
    pub viewport_y_axis: Float3,
    pub eye_position: Float3,

    pub aa_type: AntiAliasType,
    pub aa_rate: u8,

    pub width: u32,
    pub height: u32,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            spheres: Vec::new(),
            rhombohedrons: Vec::new(),
            polygons: Vec::new(),
            ellipsoids: Vec::new(),
            lights: Vec::new(),

            ambient: Float3::new(0.0, 0.0, 0.0),
            air_attenuation: Float3::new(1.0, 1.0, 1.0),

            viewport_origin: Float3::new(-0.5, -0.5, 0.0),
            viewport_x_axis: Float3::new(1.0, 0.0, 0.0),
            viewport_y_axis: Float3::new(0.0, 1.0, 0.0),
            eye_position: Float3::new(0.0, 0.0, -1.0),

            aa_type: AntiAliasType::None,
            aa_rate: 1,

            width: 800,
            height: 600,
        }
    }
}
