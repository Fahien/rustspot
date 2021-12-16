// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, convert::TryInto, error::Error, path::Path};

use gltf::Gltf;
use rayon::iter::{ParallelBridge, ParallelIterator};

use super::*;

pub struct Model {
    pub colors: HashMap<Color, Texture>,
    pub textures: Pack<Texture>,
    pub materials: Pack<Material>,
    pub primitives: Pack<Primitive>,
    pub meshes: Pack<Mesh>,
    pub nodes: Pack<Node>,
    pub directional_lights: Pack<DirectionalLight>,
    pub point_lights: Pack<PointLight>,
    pub cameras: Pack<Camera>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            colors: HashMap::new(),
            textures: Pack::new(),
            materials: Pack::new(),
            primitives: Pack::new(),
            meshes: Pack::new(),
            nodes: Pack::new(),
            directional_lights: Pack::new(),
            point_lights: Pack::new(),
            cameras: Pack::new(),
        }
    }

    /// Creates a model loading a GLTF file
    pub fn gltf<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut model = Model::new();

        let gltf = Gltf::open(&path)?;

        // Load URI buffers
        let mut uri_buffers = HashMap::new();
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    let dir = path.as_ref().parent().ok_or("Can't get parent path")?;
                    let uri = dir.join(uri);
                    let data = std::fs::read(uri)?;
                    uri_buffers.insert(buffer.index(), data);
                }
                _ => (),
            }
        }

        // Load materials
        for gmaterial in gltf.materials() {
            let mut material = Material::builder().build();
            if gmaterial
                .pbr_metallic_roughness()
                .base_color_texture()
                .is_none()
            {
                let gcolor = gmaterial.pbr_metallic_roughness().base_color_factor();
                let color = Color::rgba(
                    (gcolor[0] * 255.0) as u8,
                    (gcolor[1] * 255.0) as u8,
                    (gcolor[2] * 255.0) as u8,
                    (gcolor[3] * 255.0) as u8,
                );
                material.color = color;
                if !model.colors.contains_key(&color) {
                    let texture = Texture::pixel(color);
                    model.colors.insert(color, texture);
                }
            }

            model.materials.push(material);
        }

        // Load meshes
        for gmesh in gltf.meshes() {
            let mut primitive_handles = vec![];

            for gprimitive in gmesh.primitives() {
                let mut vertices = vec![];

                let mode = gprimitive.mode();
                assert!(mode == gltf::mesh::Mode::Triangles);

                for (semantic, accessor) in gprimitive.attributes() {
                    if semantic == gltf::mesh::Semantic::Positions {
                        let data_type = accessor.data_type();
                        assert!(data_type == gltf::accessor::DataType::F32);
                        let count = accessor.count();
                        assert!(count == 24);
                        let dimensions = accessor.dimensions();
                        assert!(dimensions == gltf::accessor::Dimensions::Vec3);

                        let view = accessor.view().unwrap();
                        let buffer = view.buffer();
                        match buffer.source() {
                            gltf::buffer::Source::Bin => unimplemented!(),
                            _ => (),
                        };

                        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
                        assert!(target == gltf::buffer::Target::ArrayBuffer);

                        let data = &uri_buffers[&buffer.index()];

                        for i in 0..count {
                            let offset = accessor.offset()
                                + view.offset()
                                + i * view.stride().unwrap_or_default();
                            assert!(offset < data.len());
                            let d = &data[offset];
                            let position = unsafe {
                                std::slice::from_raw_parts::<f32>(d as *const u8 as _, 3)
                            };
                            println!("Read position {:?}", position);

                            if vertices.len() <= i {
                                vertices.push(Vertex::new())
                            }
                            vertices[i].position = position.try_into()?;
                        }
                    } else if semantic == gltf::mesh::Semantic::Normals {
                        let data_type = accessor.data_type();
                        assert!(data_type == gltf::accessor::DataType::F32);
                        let count = accessor.count();
                        assert!(count == 24);
                        let dimensions = accessor.dimensions();
                        assert!(dimensions == gltf::accessor::Dimensions::Vec3);

                        let view = accessor.view().unwrap();
                        let buffer = view.buffer();
                        match buffer.source() {
                            gltf::buffer::Source::Bin => unimplemented!(),
                            _ => (),
                        };

                        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
                        assert!(target == gltf::buffer::Target::ArrayBuffer);

                        let data = &uri_buffers[&buffer.index()];

                        for i in 0..count {
                            let offset = accessor.offset()
                                + view.offset()
                                + i * view.stride().unwrap_or_default();
                            assert!(offset < data.len());
                            let d = &data[offset];
                            let normal = unsafe {
                                std::slice::from_raw_parts::<f32>(d as *const u8 as _, 3)
                            };
                            println!("Read normal {:?}", normal);

                            if vertices.len() <= i {
                                vertices.push(Vertex::new())
                            }
                            vertices[i].normal = normal.try_into()?;
                        }
                    }
                }

                let mut indices = vec![];
                if let Some(accessor) = gprimitive.indices() {
                    assert!(accessor.data_type() == gltf::accessor::DataType::U16);
                    let view = accessor.view().unwrap();
                    let offset = accessor.offset() + view.offset();
                    let data = &uri_buffers[&view.buffer().index()];
                    assert!(offset < data.len());
                    let d = &data[offset];
                    indices = unsafe {
                        Vec::from_raw_parts(d as *const u8 as _, accessor.count(), accessor.count())
                    };
                }

                let mut primitive = Primitive::new(vertices, indices);
                if let Some(material_id) = gprimitive.material().index() {
                    primitive.material = Some(Handle::new(material_id));
                }
                let primitive_handle = model.primitives.push(primitive);
                primitive_handles.push(primitive_handle);
            }

            let mesh = Mesh::new(primitive_handles);
            model.meshes.push(mesh);
        }

        // Load scene
        let scene = gltf.scenes().next().unwrap();
        let root = Node::builder()
            .name("Root".into())
            .children(
                scene
                    .nodes()
                    .par_bridge()
                    .map(|gchild| Handle::new(gchild.index() + 1))
                    .collect(),
            )
            .build();
        model.nodes.push(root);

        for gnode in gltf.nodes() {
            let mut node_builder = Node::builder()
                .id(gnode.index() as u32 + 1)
                .name(gnode.name().unwrap_or("Unknown").into())
                .children(
                    gnode
                        .children()
                        .par_bridge()
                        .map(|gchild| Handle::new(gchild.index() + 1))
                        .collect(),
                );

            if let Some(mesh) = gnode.mesh() {
                node_builder = node_builder.mesh(Handle::new(mesh.index()));
            }

            let node = node_builder.build();
            model.nodes.push(node);
        }

        Ok(model)
    }
}
