use crate::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
};

use super::Meshable;
use bevy_math::{
    primitives::{Capsule2d, Circle, Ellipse, Rectangle, RegularPolygon, Triangle2d, WindingOrder},
    Vec2,
};
use wgpu::PrimitiveTopology;

/// A builder used for creating a [`Mesh`] with a [`Circle`] shape.
#[derive(Clone, Copy, Debug)]
pub struct CircleMeshBuilder {
    /// The [`Circle`] shape.
    pub circle: Circle,
    /// The number of vertices used for the circle mesh.
    /// The default is `32`.
    #[doc(alias = "vertices")]
    pub resolution: usize,
}

impl Default for CircleMeshBuilder {
    fn default() -> Self {
        Self {
            circle: Circle::default(),
            resolution: 32,
        }
    }
}

impl CircleMeshBuilder {
    /// Creates a new [`CircleMeshBuilder`] from a given radius and vertex count.
    #[inline]
    pub const fn new(radius: f32, resolution: usize) -> Self {
        Self {
            circle: Circle { radius },
            resolution,
        }
    }

    /// Sets the number of vertices used for the circle mesh.
    #[inline]
    #[doc(alias = "vertices")]
    pub const fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }

    /// Builds a [`Mesh`] based on the configuration in `self`.
    pub fn build(&self) -> Mesh {
        RegularPolygon::new(self.circle.radius, self.resolution).mesh()
    }
}

impl Meshable for Circle {
    type Output = CircleMeshBuilder;

    fn mesh(&self) -> Self::Output {
        CircleMeshBuilder {
            circle: *self,
            ..Default::default()
        }
    }
}

impl From<Circle> for Mesh {
    fn from(circle: Circle) -> Self {
        circle.mesh().build()
    }
}

impl From<CircleMeshBuilder> for Mesh {
    fn from(circle: CircleMeshBuilder) -> Self {
        circle.build()
    }
}

impl Meshable for RegularPolygon {
    type Output = Mesh;

    fn mesh(&self) -> Self::Output {
        // The ellipse mesh is just a regular polygon with two radii
        Ellipse::new(self.circumcircle.radius, self.circumcircle.radius)
            .mesh()
            .resolution(self.sides)
            .build()
    }
}

impl From<RegularPolygon> for Mesh {
    fn from(polygon: RegularPolygon) -> Self {
        polygon.mesh()
    }
}

/// A builder used for creating a [`Mesh`] with an [`Ellipse`] shape.
#[derive(Clone, Copy, Debug)]
pub struct EllipseMeshBuilder {
    /// The [`Ellipse`] shape.
    pub ellipse: Ellipse,
    /// The number of vertices used for the ellipse mesh.
    /// The default is `32`.
    #[doc(alias = "vertices")]
    pub resolution: usize,
}

impl Default for EllipseMeshBuilder {
    fn default() -> Self {
        Self {
            ellipse: Ellipse::default(),
            resolution: 32,
        }
    }
}

impl EllipseMeshBuilder {
    /// Creates a new [`EllipseMeshBuilder`] from a given half width and half height and a vertex count.
    #[inline]
    pub const fn new(half_width: f32, half_height: f32, resolution: usize) -> Self {
        Self {
            ellipse: Ellipse::new(half_width, half_height),
            resolution,
        }
    }

    /// Sets the number of vertices used for the ellipse mesh.
    #[inline]
    #[doc(alias = "vertices")]
    pub const fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }

    /// Builds a [`Mesh`] based on the configuration in `self`.
    pub fn build(&self) -> Mesh {
        let mut indices = Vec::with_capacity((self.resolution - 2) * 3);
        let mut positions = Vec::with_capacity(self.resolution);
        let normals = vec![[0.0, 0.0, 1.0]; self.resolution];
        let mut uvs = Vec::with_capacity(self.resolution);

        // Add pi/2 so that there is a vertex at the top (sin is 1.0 and cos is 0.0)
        let start_angle = std::f32::consts::FRAC_PI_2;
        let step = std::f32::consts::TAU / self.resolution as f32;

        for i in 0..self.resolution {
            // Compute vertex position at angle theta
            let theta = start_angle + i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            let x = cos * self.ellipse.half_size.x;
            let y = sin * self.ellipse.half_size.y;

            positions.push([x, y, 0.0]);
            uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
        }

        for i in 1..(self.resolution as u32 - 1) {
            indices.extend_from_slice(&[0, i, i + 1]);
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }
}

impl Meshable for Ellipse {
    type Output = EllipseMeshBuilder;

    fn mesh(&self) -> Self::Output {
        EllipseMeshBuilder {
            ellipse: *self,
            ..Default::default()
        }
    }
}

impl From<Ellipse> for Mesh {
    fn from(ellipse: Ellipse) -> Self {
        ellipse.mesh().build()
    }
}

impl From<EllipseMeshBuilder> for Mesh {
    fn from(ellipse: EllipseMeshBuilder) -> Self {
        ellipse.build()
    }
}

impl Meshable for Triangle2d {
    type Output = Mesh;

    fn mesh(&self) -> Self::Output {
        let [a, b, c] = self.vertices;

        let positions = vec![[a.x, a.y, 0.0], [b.x, b.y, 0.0], [c.x, c.y, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0]; 3];

        // The extents of the bounding box of the triangle,
        // used to compute the UV coordinates of the points.
        let extents = a.min(b).min(c).abs().max(a.max(b).max(c)) * Vec2::new(1.0, -1.0);
        let uvs = vec![
            a / extents / 2.0 + 0.5,
            b / extents / 2.0 + 0.5,
            c / extents / 2.0 + 0.5,
        ];

        let is_ccw = self.winding_order() == WindingOrder::CounterClockwise;
        let indices = if is_ccw {
            Indices::U32(vec![0, 1, 2])
        } else {
            Indices::U32(vec![0, 2, 1])
        };

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(indices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl From<Triangle2d> for Mesh {
    fn from(triangle: Triangle2d) -> Self {
        triangle.mesh()
    }
}

impl Meshable for Rectangle {
    type Output = Mesh;

    fn mesh(&self) -> Self::Output {
        let [hw, hh] = [self.half_size.x, self.half_size.y];
        let positions = vec![
            [hw, hh, 0.0],
            [-hw, hh, 0.0],
            [-hw, -hh, 0.0],
            [hw, -hh, 0.0],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let uvs = vec![[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
        let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(indices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl From<Rectangle> for Mesh {
    fn from(rectangle: Rectangle) -> Self {
        rectangle.mesh()
    }
}

/// A builder used for creating a [`Mesh`] with a [`Capsule2d`] shape.
#[derive(Clone, Copy, Debug)]
pub struct Capsule2dMeshBuilder {
    /// The [`Capsule2d`] shape.
    pub capsule: Capsule2d,
    /// The number of vertices used for one hemicircle.
    /// The total number of vertices for the capsule mesh will be two times the resolution.
    ///
    /// The default is `16`.
    pub resolution: usize,
}

impl Default for Capsule2dMeshBuilder {
    fn default() -> Self {
        Self {
            capsule: Capsule2d::default(),
            resolution: 16,
        }
    }
}

impl Capsule2dMeshBuilder {
    /// Creates a new [`Capsule2dMeshBuilder`] from a given radius, length, and the number of vertices
    /// used for one hemicircle. The total number of vertices for the capsule mesh will be two times the resolution.
    #[inline]
    pub fn new(radius: f32, length: f32, resolution: usize) -> Self {
        Self {
            capsule: Capsule2d::new(radius, length),
            resolution,
        }
    }

    /// Sets the number of vertices used for one hemicircle.
    /// The total number of vertices for the capsule mesh will be two times the resolution.
    #[inline]
    pub const fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }

    /// Builds a [`Mesh`] based on the configuration in `self`.
    pub fn build(&self) -> Mesh {
        // The resolution is the number of vertices for one semicircle
        let resolution = self.resolution as u32;
        let vertex_count = 2 * self.resolution;

        // Six extra indices for the two triangles between the hemicircles
        let mut indices = Vec::with_capacity((self.resolution - 2) * 2 * 3 + 6);
        let mut positions = Vec::with_capacity(vertex_count);
        let normals = vec![[0.0, 0.0, 1.0]; vertex_count];
        let mut uvs = Vec::with_capacity(vertex_count);

        let radius = self.capsule.radius;
        let step = std::f32::consts::TAU / vertex_count as f32;

        // If the vertex count is even, offset starting angle of top semicircle by half a step
        // to position the vertices evenly.
        let start_angle = if vertex_count % 2 == 0 {
            step / 2.0
        } else {
            0.0
        };

        // How much the hemicircle radius is of the total half-height of the capsule.
        // This is used to prevent the UVs from stretching between the hemicircles.
        let radius_frac = self.capsule.radius / (self.capsule.half_length + self.capsule.radius);

        // Create top semicircle
        for i in 0..resolution {
            // Compute vertex position at angle theta
            let theta = start_angle + i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            let (x, y) = (cos * radius, sin * radius + self.capsule.half_length);

            positions.push([x, y, 0.0]);
            uvs.push([0.5 * (cos + 1.0), radius_frac * (1.0 - 0.5 * (sin + 1.0))]);
        }

        // Add top semicircle indices
        for i in 1..resolution - 1 {
            indices.extend_from_slice(&[0, i, i + 1]);
        }

        // Add indices for top left triangle of the part between the hemicircles
        indices.extend_from_slice(&[0, resolution - 1, resolution]);

        // Create bottom semicircle
        for i in resolution..vertex_count as u32 {
            // Compute vertex position at angle theta
            let theta = start_angle + i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            let (x, y) = (cos * radius, sin * radius - self.capsule.half_length);

            positions.push([x, y, 0.0]);
            uvs.push([0.5 * (cos + 1.0), 1.0 - radius_frac * 0.5 * (sin + 1.0)]);
        }

        // Add bottom semicircle indices
        for i in 1..resolution - 1 {
            indices.extend_from_slice(&[resolution, resolution + i, resolution + i + 1]);
        }

        // Add indices for bottom right triangle of the part between the hemicircles
        indices.extend_from_slice(&[resolution, vertex_count as u32 - 1, 0]);

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }
}

impl Meshable for Capsule2d {
    type Output = Capsule2dMeshBuilder;

    fn mesh(&self) -> Self::Output {
        Capsule2dMeshBuilder {
            capsule: *self,
            ..Default::default()
        }
    }
}

impl From<Capsule2d> for Mesh {
    fn from(capsule: Capsule2d) -> Self {
        capsule.mesh().build()
    }
}

impl From<Capsule2dMeshBuilder> for Mesh {
    fn from(capsule: Capsule2dMeshBuilder) -> Self {
        capsule.build()
    }
}
