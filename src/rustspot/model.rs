// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    convert::TryInto,
    error::Error,
    path::{Path, PathBuf},
};

use gltf::Gltf;
use nalgebra as na;
use rayon::iter::{ParallelBridge, ParallelIterator};

use super::*;

fn data_type_as_size(data_type: gltf::accessor::DataType) -> usize {
    match data_type {
        gltf::accessor::DataType::I8 => 1,
        gltf::accessor::DataType::U8 => 1,
        gltf::accessor::DataType::I16 => 2,
        gltf::accessor::DataType::U16 => 2,
        gltf::accessor::DataType::U32 => 4,
        gltf::accessor::DataType::F32 => 4,
    }
}

fn dimensions_as_size(dimensions: gltf::accessor::Dimensions) -> usize {
    match dimensions {
        gltf::accessor::Dimensions::Scalar => 1,
        gltf::accessor::Dimensions::Vec2 => 2,
        gltf::accessor::Dimensions::Vec3 => 3,
        gltf::accessor::Dimensions::Vec4 => 4,
        gltf::accessor::Dimensions::Mat2 => 4,
        gltf::accessor::Dimensions::Mat3 => 9,
        gltf::accessor::Dimensions::Mat4 => 16,
    }
}

fn get_stride(accessor: &gltf::Accessor) -> usize {
    if let Some(view) = accessor.view() {
        if let Some(stride) = view.stride() {
            return stride;
        }
    }

    data_type_as_size(accessor.data_type()) * dimensions_as_size(accessor.dimensions())
}

pub struct ModelBuilder {
    uri_buffers: Vec<Vec<u8>>,
    parent_dir: PathBuf,
    gltf: Gltf,
}

impl ModelBuilder {
    /// Creates a model loading a GLTF file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let ret = Self {
            uri_buffers: vec![],
            parent_dir: path
                .as_ref()
                .parent()
                .ok_or("Failed to get parent directory")?
                .into(),
            gltf: Gltf::open(path)?,
        };
        Ok(ret)
    }

    fn load_uri_buffers(&mut self) -> Result<(), Box<dyn Error>> {
        for buffer in self.gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    let uri = self.parent_dir.join(uri);
                    let data = std::fs::read(uri)?;
                    assert!(buffer.index() == self.uri_buffers.len());
                    self.uri_buffers.push(data);
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }

    fn get_data_start(&self, accessor: &gltf::Accessor) -> &[u8] {
        let view = accessor.view().unwrap();
        let view_len = view.length();

        let buffer = view.buffer();
        if let gltf::buffer::Source::Bin = buffer.source() {
            unimplemented!()
        }

        let view_offset = view.offset();
        let accessor_offset = accessor.offset();
        let offset = accessor_offset + view_offset;
        assert!(offset < buffer.length());

        let data = &self.uri_buffers[buffer.index()];
        &data[offset..offset + view_len]
    }

    pub fn load_materials(&mut self, model: &mut Model) -> Result<(), Box<dyn Error>> {
        for gmaterial in self.gltf.materials() {
            let mut material = Material::builder().shader(Shaders::LIGHTSHADOW).build();

            let pbr = gmaterial.pbr_metallic_roughness();
            if let Some(gtexture) = pbr.base_color_texture() {
                match gtexture.texture().source().source() {
                    gltf::image::Source::Uri { uri, .. } => {
                        let uri = self.parent_dir.join(uri);
                        let texture_handle = if let Some((index, _)) =
                            model.textures.iter().enumerate().find(|(_, texture)| {
                                texture.path.is_some() && *texture.path.as_ref().unwrap() == uri
                            }) {
                            Handle::new(index)
                        } else {
                            let texture = Texture::builder().path(uri).build()?;
                            model.textures.push(texture)
                        };
                        material.texture = Some(texture_handle);
                    }
                    _ => unimplemented!(),
                }
            } else {
                let gcolor = gmaterial.pbr_metallic_roughness().base_color_factor();
                let color = Color::rgba(
                    (gcolor[0] * 255.0) as u8,
                    (gcolor[1] * 255.0) as u8,
                    (gcolor[2] * 255.0) as u8,
                    (gcolor[3] * 255.0) as u8,
                );
                material.color = color;
                if !model.colors.contains_key(&color) {
                    let texture = Texture::builder().data(color.as_slice()).build()?;
                    model.colors.insert(color, texture);
                }
            }

            model.materials.push(material);
        }

        Ok(())
    }

    pub fn build(&mut self) -> Result<Model, Box<dyn Error>> {
        let mut model = Model::new();

        self.load_uri_buffers()?;
        self.load_materials(&mut model);
        self.load_meshes(&mut model)?;

        // Load scene
        let scene = self.gltf.scenes().next().unwrap();
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
        // Root is always at index 0
        model.nodes.push(root);

        // Load nodes
        for gnode in self.gltf.nodes() {
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

            let transform = gnode.transform().decomposed();

            let translation = &transform.0;
            let translation = na::Translation3::new(translation[0], translation[1], translation[2]);
            node_builder = node_builder.translation(translation);

            // xyzw
            let rotation = &transform.1;
            let rotation = na::UnitQuaternion::from_quaternion(na::Quaternion::new(
                rotation[3],
                rotation[0],
                rotation[1],
                rotation[2],
            ));
            node_builder = node_builder.rotation(rotation);

            let scale = &transform.2;
            let scale = na::Vector3::new(scale[0], scale[1], scale[2]);
            node_builder = node_builder.scale(scale);

            if let Some(mesh) = gnode.mesh() {
                node_builder = node_builder.mesh(Handle::new(mesh.index()));
            }

            let node = node_builder.build();
            model.nodes.push(node);
        }

        Ok(model)
    }

    fn load_meshes(&self, model: &mut Model) -> Result<(), Box<dyn Error>> {
        for gmesh in self.gltf.meshes() {
            let mut primitive_handles = vec![];

            for gprimitive in gmesh.primitives() {
                let mut vertices = vec![];

                let mode = gprimitive.mode();
                assert!(mode == gltf::mesh::Mode::Triangles);

                for (semantic, accessor) in gprimitive.attributes() {
                    match semantic {
                        gltf::mesh::Semantic::Positions => {
                            self.load_positions(&mut vertices, &accessor)?
                        }
                        gltf::mesh::Semantic::Normals => {
                            self.load_normals(&mut vertices, &accessor)?
                        }
                        gltf::mesh::Semantic::TexCoords(_) => {
                            self.load_tex_coords(&mut vertices, &accessor)?
                        }
                        _ => unimplemented!(),
                    }
                }

                let mut indices = vec![];
                if let Some(accessor) = gprimitive.indices() {
                    let data_type = accessor.data_type();
                    assert!(data_type == gltf::accessor::DataType::U16);
                    let data = self.get_data_start(&accessor);
                    let d = &data[0];
                    let length = accessor.count();
                    let slice: &[u16] =
                        unsafe { std::slice::from_raw_parts(d as *const u8 as _, length) };
                    indices = Vec::from(slice);
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

        Ok(())
    }

    fn load_positions(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec3);

        let view = accessor.view().unwrap();

        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let position = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 3) };

            if vertices.len() <= i {
                vertices.push(Vertex::new())
            }
            vertices[i].position = position.try_into()?;
        }

        Ok(())
    }

    fn load_normals(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec3);

        let view = accessor.view().unwrap();
        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let normal = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 3) };

            if vertices.len() <= i {
                vertices.push(Vertex::new())
            }
            vertices[i].normal = normal.try_into()?;
        }

        Ok(())
    }

    fn load_tex_coords(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec2);

        let view = accessor.view().unwrap();
        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let tex_coords = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 2) };

            if vertices.len() <= i {
                vertices.push(Vertex::new())
            }
            vertices[i].tex_coords = tex_coords.try_into()?;
        }

        Ok(())
    }
}

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
    pub fn builder<P: AsRef<Path>>(path: P) -> Result<ModelBuilder, Box<dyn Error>> {
        ModelBuilder::new(path)
    }

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
}
