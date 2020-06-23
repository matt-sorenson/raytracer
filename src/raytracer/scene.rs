use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

use super::shapes::*;

use super::Float3;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum AntiAliasType {
    None,
    SuperSample,
    MonteCarlo,
}

#[derive(Serialize, Deserialize)]
pub struct Scene {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spheres: Vec<Sphere>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rhombohedrons: Vec<Rhombohedron>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub polygons: Vec<Polygon>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ellipsoids: Vec<Ellipsoid>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
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
    pub fn from_file(filename: &str) -> Scene {
        let file = File::open(filename).expect("Failed to open file");
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).expect("Failed to deserialize json")
    }

    #[allow(dead_code)]
    pub fn to_file(&self, filename: &str) {
        let file = File::create(filename).expect("Failed to create file.");
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self).expect("Failed to write to file.");
    }
}
