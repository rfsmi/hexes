use std::{collections::HashMap, f32::consts::PI, iter::zip};

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use itertools::*;

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
}

impl MeshBuilder {
    fn add_triangle(&mut self, points: [[f32; 3]; 3]) {
        let [a, b, c] = points.map(Vec3::from_array);
        let normal = (b - a).cross(c - a).normalize();
        for p in points {
            self.positions.push(Vec3::from_array(p));
            self.normals.push(normal);
        }
    }
}

impl From<MeshBuilder> for Mesh {
    fn from(value: MeshBuilder) -> Self {
        let mut positions = vec![];
        let mut normals = vec![];
        let mut indices = vec![];
        let mut cache = HashMap::new();
        for (p, n) in zip(value.positions, value.normals) {
            let (p, n) = ((p * 1e6).round() / 1e6, (n * 1e6).round() / 1e6);
            let i = *cache
                .entry((p.to_string(), n.to_string()))
                .or_insert_with(|| {
                    positions.push(p);
                    normals.push(n);
                    positions.len() as u32 - 1
                });
            indices.push(i);
        }
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
    }
}

pub fn generate(n_sides: u32) -> Mesh {
    let mut builder = MeshBuilder::default();
    let outer_points = (0..n_sides)
        .flat_map(|i| {
            let angle = i as f32 * 2.0 * PI / n_sides as f32;
            let (x, z) = (angle.cos(), angle.sin());
            [[x, 0.5, z], [x, -0.5, z]]
        })
        .collect_vec();
    for (t1, b1, t2, b2) in outer_points.iter().copied().circular_tuple_windows() {
        // The side face, viewed from outside the mesh:
        //    t2---t1
        //    |   / |
        //    | /   |
        //    b2---b1
        builder.add_triangle([t1, b2, b1]);
        builder.add_triangle([t1, t2, b2]);
    }
    let mut outer_points = outer_points.into_iter();
    let (t0, b0) = outer_points.next_tuple().unwrap();
    for (t1, b1, t2, b2) in outer_points.tuple_windows() {
        // The top triangle, viewed from above:
        //      t0---t1
        //     /  \    \
        //    /      \  \
        //             t2
        builder.add_triangle([t0, t2, t1]);
        // The bottom triangle, viewed from below:
        //      t0---t2
        //     /    /  \
        //    /  /      \
        //    t1
        builder.add_triangle([b0, b1, b2]);
    }
    builder.into()
}
