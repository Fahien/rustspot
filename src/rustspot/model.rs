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

fn data_type_as_gl(data_type: gltf::accessor::DataType) -> gl::types::GLenum {
    match data_type {
        gltf::accessor::DataType::I8 => todo!(),
        gltf::accessor::DataType::U8 => gl::UNSIGNED_BYTE,
        gltf::accessor::DataType::I16 => todo!(),
        gltf::accessor::DataType::U16 => gl::UNSIGNED_SHORT,
        gltf::accessor::DataType::U32 => gl::UNSIGNED_INT,
        gltf::accessor::DataType::F32 => todo!(),
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
        let mut timer = Timer::new();

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

        println!("Buffers loaded ({}s)", timer.get_delta().as_secs_f32());
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
        let end_offset = view_offset + view_len;
        assert!(end_offset <= buffer.length());

        let data = &self.uri_buffers[buffer.index()];
        &data[offset..end_offset]
    }

    pub fn load_textures(&mut self, model: &mut Model) {
        let mut timer = Timer::new();

        // Let us load textures first
        let texture_builders: Vec<TextureBuilder> = self
            .gltf
            .images()
            .enumerate()
            .par_bridge()
            .map(|(i, image)| {
                match image.source() {
                    gltf::image::Source::View { .. } => todo!(),
                    gltf::image::Source::Uri { uri, .. } => {
                        // Join gltf parent dir to URI
                        let path = self.parent_dir.join(uri);
                        Texture::builder().id(i as u32).path(path)
                    }
                }
            })
            .collect();

        println!(
            "Loaded images from file ({}s)",
            timer.get_delta().as_secs_f32()
        );

        // This can not be done in parallel as OpenGL is not multithread-friendly
        let mut textures: Vec<Texture> = texture_builders
            .into_iter()
            .map(|builder| builder.build().unwrap())
            .collect();

        textures.sort_by_key(|tex| tex.id);
        model.textures = Pack::from(textures);

        println!(
            "Loaded images to GPU ({}s)",
            timer.get_delta().as_secs_f32()
        );
    }

    pub fn load_texture(
        &self,
        textures: &Pack<Texture>,
        texture: &gltf::Texture,
    ) -> Option<Handle<Texture>> {
        match texture.source().source() {
            gltf::image::Source::Uri { uri, .. } => {
                let uri = self.parent_dir.join(uri);

                // Find texture by path
                let (texture_id, _) = textures
                    .iter()
                    .enumerate()
                    .find(|(_, texture)| {
                        texture.path.is_some() && *texture.path.as_ref().unwrap() == uri
                    })
                    .unwrap();

                let texture_handle = Handle::new(texture_id);

                return Some(texture_handle);
            }
            _ => unimplemented!(),
        }
    }

    pub fn load_materials(
        &mut self,
        textures: &Pack<Texture>,
        colors: &mut HashMap<Color, Texture>,
        materials: &mut Pack<Material>,
    ) -> Result<(), Box<dyn Error>> {
        let _ = ScopedTimer::new("Materials loaded");

        for gmaterial in self.gltf.materials() {
            let mut material = Material::builder().shader(Shaders::LIGHTSHADOW).build();

            let pbr = gmaterial.pbr_metallic_roughness();

            // Load albedo
            if let Some(gtexture) = pbr.base_color_texture() {
                material.texture = self.load_texture(textures, &gtexture.texture());
            } else {
                let gcolor = gmaterial.pbr_metallic_roughness().base_color_factor();
                let color = Color::rgba(
                    (gcolor[0] * 255.0) as u8,
                    (gcolor[1] * 255.0) as u8,
                    (gcolor[2] * 255.0) as u8,
                    (gcolor[3] * 255.0) as u8,
                );
                material.color = color;
                if !colors.contains_key(&color) {
                    let texture = Texture::builder().data(color.as_slice()).build()?;
                    colors.insert(color, texture);
                }
            }

            // Load normal map
            if let Some(gtexture) = gmaterial.normal_texture() {
                material.normals = self.load_texture(&textures, &gtexture.texture());
            }

            // Load ambient occlusion texture
            if let Some(gtexture) = gmaterial.occlusion_texture() {
                material.occlusion = self.load_texture(&textures, &gtexture.texture());
            }

            // Load metallic rougness texture
            if let Some(gtexture) = pbr.metallic_roughness_texture() {
                material.metallic_roughness = self.load_texture(&textures, &gtexture.texture());
            }

            // Determines shader based on textures available
            material.shader = if let Some(metallic_roughness) = material.metallic_roughness {
                if let Some(occlusion) = material.occlusion {
                    if occlusion.id == metallic_roughness.id {
                        Shaders::LIGHTSHADOWMRO
                    } else {
                        Shaders::LIGHTSHADOWMROCCLUSION
                    }
                } else {
                    Shaders::LIGHTSHADOWMR
                }
            } else {
                Shaders::LIGHTSHADOW
            };

            material.metallic = pbr.metallic_factor();
            material.roughness = pbr.roughness_factor();

            materials.push(material);
        }

        Ok(())
    }

    pub fn build(&mut self) -> Result<Model, Box<dyn Error>> {
        let mut model = Model::new();

        self.load_uri_buffers()?;
        self.load_textures(&mut model);
        self.load_materials(&model.textures, &mut model.colors, &mut model.materials)?;
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

                // Load normals first, so we can process tangents later
                for (semantic, accessor) in gprimitive.attributes() {
                    if semantic == gltf::mesh::Semantic::Normals {
                        self.load_normals(&mut vertices, &accessor)?;
                    }
                }

                for (semantic, accessor) in gprimitive.attributes() {
                    match semantic {
                        gltf::mesh::Semantic::Positions => {
                            self.load_positions(&mut vertices, &accessor)?
                        }
                        gltf::mesh::Semantic::TexCoords(_) => {
                            self.load_tex_coords(&mut vertices, &accessor)?
                        }
                        gltf::mesh::Semantic::Tangents => {
                            self.load_tangents(&mut vertices, &accessor)?
                        }
                        _ => println!("Semantic not implemented {:?}", semantic),
                    }
                }

                let mut indices = vec![];
                let mut index_type = gl::UNSIGNED_BYTE;
                if let Some(accessor) = gprimitive.indices() {
                    let data_type = accessor.data_type();
                    index_type = data_type_as_gl(data_type);

                    // Data type can vary
                    let data = self.get_data_start(&accessor);
                    let d = &data[0];
                    let length = accessor.count() * data_type_as_size(data_type);
                    // Use bytes regardless of the index data type
                    let slice: &[u8] =
                        unsafe { std::slice::from_raw_parts(d as *const u8 as _, length) };
                    indices = Vec::from(slice);
                }

                let material = gprimitive.material().index().map(|id| Handle::new(id));

                let primitive = Primitive::builder()
                    .vertices(vertices)
                    .indices(indices)
                    .index_type(index_type)
                    .material(material)
                    .build();
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
            vertices[i].normal[0] = normal[0];
            vertices[i].normal[1] = normal[1];
            vertices[i].normal[2] = normal[2];
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

    fn load_tangents(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec4);

        let view = accessor.view().unwrap();
        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let tangent = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 4) };

            if vertices.len() <= i {
                vertices.push(Vertex::new())
            }
            vertices[i].tangent[0] = tangent[0];
            vertices[i].tangent[1] = tangent[1];
            vertices[i].tangent[2] = tangent[2];

            // Compute bitangent as for glTF 2.0 spec
            vertices[i].bitangent = vertices[i].normal.cross(&vertices[i].tangent) * tangent[3];
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
