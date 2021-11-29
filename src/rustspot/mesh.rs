// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct MeshRes {
    pub vbo: Vbo,
    pub ebo: Ebo,
    pub vao: Vao,
}

impl MeshRes {
    pub fn new() -> Self {
        let vbo = Vbo::new();
        let ebo = Ebo::new();
        let vao = Vao::new();

        Self { vbo, ebo, vao }
    }

    pub fn from(vertices: &[Vertex], indices: &Vec<u16>) -> Self {
        let mut res = MeshRes::new();

        res.vao.bind();
        res.vbo.upload(&vertices);
        res.ebo.upload(&indices);

        let stride = std::mem::size_of::<Vertex>() as i32;

        // These should follow Vao, Vbo, Ebo
        unsafe {
            // Position
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as _);
            gl::EnableVertexAttribArray(0);

            // Color
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(1);

            // Texture coordinates
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (6 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(2);

            // Normal
            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::TRUE,
                stride,
                (8 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(3);
        }

        res
    }

    pub fn bind(&self) {
        self.vao.bind();
    }
}

/// Geometry to be rendered with a given material
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    /// None means default material
    pub material: Option<Handle<Material>>,

    // Res could be computed on the fly, but we would need to hash both vertices and indices,
    // therefore we store it here and it is responsibility of the scene builder to avoid an
    // explosion of primitive resources at run-time.
    res: MeshRes,
}

impl Primitive {
    /// Creates a new primitive
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        let res = MeshRes::from(&vertices, &indices);

        Self {
            vertices,
            indices,
            material: None,
            res,
        }
    }

    /// Returns a new unit triangle primitive
    pub fn triangle(material: Handle<Material>) -> Self {
        let vertices = vec![
            Vertex {
                position: [-1.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.5, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
        ];

        let indices = vec![0, 1, 2];

        let res = MeshRes::from(&vertices, &indices);

        Self {
            vertices,
            indices,
            material: Some(material),
            res,
        }
    }

    /// Returns a new primitive quad with side length 1 centered at the origin
    pub fn quad(material: Handle<Material>) -> Self {
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];

        let res = MeshRes::from(&vertices, &indices);

        Self {
            vertices,
            indices,
            material: Some(material),
            res,
        }
    }

    pub fn cube(material: Handle<Material>) -> Self {
        let mut vertices = vec![Vertex::new(); 24];

        let (tex_width, tex_height) = (4.0, 4.0);

        // Front
        vertices[0].position = [-0.5, -0.5, 0.5];
        vertices[0].tex_coords = [1.0 / tex_width, 1.0 / tex_height];
        vertices[0].normal = [0.0, 0.0, 1.0];
        vertices[1].position = [0.5, -0.5, 0.5];
        vertices[1].tex_coords = [2.0 / tex_width, 1.0 / tex_height];
        vertices[1].normal = [0.0, 0.0, 1.0];
        vertices[2].position = [0.5, 0.5, 0.5];
        vertices[2].tex_coords = [2.0 / tex_width, 2.0 / tex_height];
        vertices[2].normal = [0.0, 0.0, 1.0];
        vertices[3].position = [-0.5, 0.5, 0.5];
        vertices[3].tex_coords = [1.0 / tex_width, 2.0 / tex_height];
        vertices[3].normal = [0.0, 0.0, 1.0];

        // Right
        vertices[4].position = [0.5, -0.5, 0.5];
        vertices[4].normal = [1.0, 0.0, 0.0];
        vertices[4].tex_coords = [2.0 / tex_width, 1.0 / tex_height];
        vertices[5].position = [0.5, -0.5, -0.5];
        vertices[5].normal = [1.0, 0.0, 0.0];
        vertices[5].tex_coords = [3.0 / tex_width, 1.0 / tex_height];
        vertices[6].position = [0.5, 0.5, -0.5];
        vertices[6].normal = [1.0, 0.0, 0.0];
        vertices[6].tex_coords = [3.0 / tex_width, 2.0 / tex_height];
        vertices[7].position = [0.5, 0.5, 0.5];
        vertices[7].normal = [1.0, 0.0, 0.0];
        vertices[7].tex_coords = [2.0 / tex_width, 2.0 / tex_height];

        // Back
        vertices[8].position = [0.5, -0.5, -0.5];
        vertices[8].normal = [0.0, 0.0, -1.0];
        vertices[8].tex_coords = [3.0 / tex_width, 1.0 / tex_height];
        vertices[9].position = [-0.5, -0.5, -0.5];
        vertices[9].normal = [0.0, 0.0, -1.0];
        vertices[9].tex_coords = [4.0 / tex_width, 1.0 / tex_height];
        vertices[10].position = [-0.5, 0.5, -0.5];
        vertices[10].normal = [0.0, 0.0, -1.0];
        vertices[10].tex_coords = [4.0 / tex_width, 2.0 / tex_height];
        vertices[11].position = [0.5, 0.5, -0.5];
        vertices[11].normal = [0.0, 0.0, -1.0];
        vertices[11].tex_coords = [3.0 / tex_width, 2.0 / tex_height];

        // Left
        vertices[12].position = [-0.5, -0.5, -0.5];
        vertices[12].normal = [-1.0, 0.0, 0.0];
        vertices[12].tex_coords = [0.0, 1.0 / tex_height];
        vertices[13].position = [-0.5, -0.5, 0.5];
        vertices[13].normal = [-1.0, 0.0, 0.0];
        vertices[13].tex_coords = [1.0 / tex_width, 1.0 / tex_height];
        vertices[14].position = [-0.5, 0.5, 0.5];
        vertices[14].normal = [-1.0, 0.0, 0.0];
        vertices[14].tex_coords = [1.0 / tex_width, 2.0 / tex_height];
        vertices[15].position = [-0.5, 0.5, -0.5];
        vertices[15].normal = [-1.0, 0.0, 0.0];
        vertices[15].tex_coords = [0.0, 2.0 / tex_height];

        // Top
        vertices[16].position = [-0.5, 0.5, 0.5];
        vertices[16].normal = [0.0, 1.0, 0.0];
        vertices[16].tex_coords = [1.0 / tex_width, 2.0 / tex_height];
        vertices[17].position = [0.5, 0.5, 0.5];
        vertices[17].normal = [0.0, 1.0, 0.0];
        vertices[17].tex_coords = [2.0 / tex_width, 2.0 / tex_height];
        vertices[18].position = [0.5, 0.5, -0.5];
        vertices[18].normal = [0.0, 1.0, 0.0];
        vertices[18].tex_coords = [2.0 / tex_width, 3.0 / tex_height];
        vertices[19].position = [-0.5, 0.5, -0.5];
        vertices[19].normal = [0.0, 1.0, 0.0];
        vertices[19].tex_coords = [1.0 / tex_width, 3.0 / tex_height];

        // Bottom
        vertices[20].position = [-0.5, -0.5, -0.5];
        vertices[20].normal = [0.0, -1.0, 0.0];
        vertices[20].tex_coords = [1.0 / tex_width, 0.0];
        vertices[21].position = [0.5, -0.5, -0.5];
        vertices[21].normal = [0.0, -1.0, 0.0];
        vertices[21].tex_coords = [2.0 / tex_width, 0.0];
        vertices[22].position = [0.5, -0.5, 0.5];
        vertices[22].normal = [0.0, -1.0, 0.0];
        vertices[22].tex_coords = [2.0 / tex_width, 1.0 / tex_height];
        vertices[23].position = [-0.5, -0.5, 0.5];
        vertices[23].normal = [0.0, -1.0, 0.0];
        vertices[23].tex_coords = [1.0 / tex_width, 1.0 / tex_height];

        let indices = vec![
            0, 1, 2, 0, 2, 3, // front face
            4, 5, 6, 4, 6, 7, // right
            8, 9, 10, 8, 10, 11, // back
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // top
            20, 21, 22, 20, 22, 23, // bottom
        ];

        let res = MeshRes::from(&vertices, &indices);

        Self {
            vertices,
            indices,
            material: Some(material),
            res,
        }
    }

    /// This function is going to bind only this primitive's VAO. We do not bind the
    /// primitives' material here because we expect the renderer has already bound it.
    pub fn bind(&self) {
        self.res.bind();
    }

    pub fn draw(&self) {
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as _,
                gl::UNSIGNED_SHORT,
                0 as _,
            );
        }
    }
}

/// A mesh is an array of primitives to be rendered. A node can contain
/// one mesh, and a node's transform places the mesh in the scene
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Handle<Primitive>>,
}

impl Mesh {
    pub fn new(primitives: Vec<Handle<Primitive>>) -> Self {
        Self {
            name: String::new(),
            primitives,
        }
    }
}
