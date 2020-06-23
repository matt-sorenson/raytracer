extern crate nalgebra as na;

use super::Float3;
pub type Float3x3 = na::Matrix3<f64>;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub diffuse: Float3,
    pub specular_coefficient: f64,
    pub specular_power: f64,
    pub attenuation: Float3,
    pub electric_permittivity: f64,
    pub magnetic_permeability: f64,
    pub index_of_refraction: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Float3,
    pub direction: Float3,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Float3,
    pub radius: f64,

    pub material: Material,
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub normal: Float3,
    pub point: Float3,
}

// 'Box'
#[derive(Debug, Copy, Clone)]
pub struct Rhombohedron {
    pub planes: [Plane; 6],

    pub material: Material,
}

impl Rhombohedron {
    pub fn from_corner_and_edges(
        corner: Float3,
        length: Float3,
        width: Float3,
        height: Float3,
        material: Material,
    ) -> Rhombohedron {
        let plane_0 = Plane {
            point: corner,
            normal: length.cross(&height).normalize(),
        };
        let plane_1 = Plane {
            point: corner + width,
            normal: -plane_0.normal,
        };
        let plane_2 = Plane {
            point: corner,
            normal: height.cross(&width).normalize(),
        };
        let plane_3 = Plane {
            point: corner + length,
            normal: -plane_2.normal,
        };
        let plane_4 = Plane {
            point: corner,
            normal: width.cross(&length).normalize(),
        };
        let plane_5 = Plane {
            point: corner + height,
            normal: -plane_4.normal,
        };

        Rhombohedron {
            planes: [plane_0, plane_1, plane_2, plane_3, plane_4, plane_5],
            material,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub vertices: [Float3; 3],
    pub edges: [Float3; 2],
    pub normal: Float3,
}

impl Triangle {
    pub fn from_vertices(vertices: [Float3; 3]) -> Self {
        let edges: [Float3; 2] = [vertices[1] - vertices[0], vertices[2] - vertices[0]];
        let normal = edges[0].cross(&edges[1]).normalize();

        let out = Triangle {
            vertices,
            edges,
            normal,
        };

        out
    }

    pub fn contains(&self, point: &Float3) -> bool {
        let aa = self.edges[0].dot(&self.edges[0]);
        let bb = self.edges[1].dot(&self.edges[1]);
        let ab = self.edges[0].dot(&self.edges[1]);

        let pc = point - self.vertices[0];
        let inv_det = 1.0 / ((aa * bb) - (ab * ab));

        let x_ = pc.dot(&self.edges[0]);
        let y_ = pc.dot(&self.edges[1]);

        let x = inv_det * ((bb * x_) + (-ab * y_));
        let y = inv_det * ((-ab * x_) + (aa * y_));

        (x > 0.0) && (y > 0.0) && ((x + y) < 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub triangles: Vec<Triangle>,
    pub plane: Plane,

    pub material: Material,
}

impl Polygon {
    pub fn from_vertices(vertices: Vec<Float3>, material: Material) -> Self {
        let mut triangles = Vec::new();

        for i in 1..(vertices.len() - 1) {
            triangles.push(Triangle::from_vertices([
                vertices[0],
                vertices[i],
                vertices[i + 1],
            ]));
        }

        let plane = Plane {
            normal: triangles[0].normal,
            point: vertices[0],
        };

        Polygon {
            triangles,
            plane,
            material,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ellipsoid {
    pub center: Float3,
    pub inverse: Float3x3,
    pub inverse_transpose: Float3x3,

    pub material: Material,
}

impl Ellipsoid {
    pub fn new(center: Float3, semiaxes: [Float3; 3], material: Material) -> Self {
        let m = Float3x3::from_columns(&semiaxes);

        let inverse = m.try_inverse().expect("Ellipsoid transform non-invertable");
        let inverse_transpose = inverse.transpose();

        Ellipsoid { center, inverse, inverse_transpose, material }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub center: Float3,
    pub radius: f64,
    pub color: Float3,
}
