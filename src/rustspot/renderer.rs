// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

use nalgebra as na;
use std::collections::HashMap;

pub struct Renderer {
    /// Delta time used as a uniform in shaders
    pub delta: f32,

    gui_res: GuiRes,

    custom_shaders: Vec<Box<dyn CustomShader>>,

    /// List of shader enums to bind with materials referring to them.
    shaders: HashMap<Shaders, Vec<usize>>,

    /// Node with the directional light to use for rendering
    directional_light: Handle<Node>,

    /// List of point light handles to use while drawing the scene paired with the node to use
    point_lights: Vec<Handle<Node>>,

    /// List of camera handles to use while drawing the scene paired with the node to use
    pub cameras: Vec<(Handle<Camera>, Handle<Node>)>,

    /// List of material handles to bind with primitives referring to them.
    materials: HashMap<usize, Vec<usize>>,

    /// List of primitive handles to draw with nodes referring to them.
    /// Together with nodes, we store their transform matrix computed during the scene graph traversal.
    primitives: HashMap<usize, HashMap<usize, na::Matrix4<f32>>>,

    pub read_depth_program: ShaderProgram,
    pub read_color_program: ShaderProgram,

    /// Orthographic camera and node for camera
    pub screen_camera: Camera,
    pub screen_node: Node,

    /// Quad for rendering texture to screen
    pub quad_primitive: Primitive,
    pub quad_node: Node,

    /// Used for shadows
    pub light_space: na::Matrix4<f32>,
    /// Handle to the shadowmap
    pub shadow_map: u32,

    pub sky: Sky,
}

impl Renderer {
    pub fn new(profile: sdl2::video::GLProfile, fonts: &mut imgui::FontAtlasRefMut) -> Renderer {
        let read_depth_program = ShaderProgram::open(
            profile,
            "res/shader/default.vert.glsl",
            "res/shader/read-depth.frag.glsl",
        );

        let read_color_program = ShaderProgram::open(
            profile,
            "res/shader/default.vert.glsl",
            "res/shader/read-color.frag.glsl",
        );

        let screen_camera = Camera::orthographic(1, 1);
        let mut screen_node = Node::new();
        screen_node.trs.translate(0.0, 0.0, 1.0);

        // Create a unit quad to present offscreen texture
        let quad_primitive = Primitive::quad(Handle::none());
        let quad_node = Node::new();

        let sky = Sky::new(profile);

        Renderer {
            delta: 0.0,
            gui_res: GuiRes::new(profile, fonts),
            custom_shaders: create_shaders(profile),
            shaders: HashMap::new(),
            directional_light: Handle::none(),
            point_lights: Vec::new(),
            cameras: Vec::new(),
            materials: HashMap::new(),
            primitives: HashMap::new(),

            read_depth_program,
            read_color_program,

            screen_camera,
            screen_node,

            quad_primitive,
            quad_node,

            light_space: na::Matrix4::identity(),
            shadow_map: 0,
            sky,
        }
    }

    /// Draw does not render immediately, instead it creates a list of mesh resources.
    /// At the same time it computes transform matrices for each node to be bound later on.
    pub fn draw(&mut self, model: &Model, node_handle: Handle<Node>, transform: &na::Matrix4<f32>) {
        // Precompute transform matrix
        let node = &model.nodes[node_handle.id];
        let temp_transform = transform * node.trs.get_matrix();

        // The current node
        let node = model.nodes.get(node_handle).unwrap();

        // Here we add this to a list of nodes that should be rendered
        let mesh = node.mesh;
        if let Some(mesh) = model.meshes.get(mesh) {
            for &primitive_handle in mesh.primitives.iter() {
                let primitive = model.primitives.get(primitive_handle).unwrap();
                let material_handle = primitive.material.unwrap();
                let material = model.materials.get(material_handle).unwrap();

                // Store this association shader program, material
                let key = material.shader;
                if let Some(shader_materials) = self.shaders.get_mut(&key) {
                    // Add this material id to the value list of not already there
                    if !shader_materials.contains(&material_handle.id) {
                        shader_materials.push(material_handle.id);
                    }
                } else {
                    // Create a new entry (shader, material)
                    self.shaders.insert(key, vec![material_handle.id]);
                }

                // Store this association material, primitive
                let key = material_handle.id;
                // Check if an entry already exists
                if let Some(material_primitives) = self.materials.get_mut(&key) {
                    // Add this primitive id to the value list if not already there
                    if !material_primitives.contains(&primitive_handle.id) {
                        material_primitives.push(primitive_handle.id);
                    }
                } else {
                    // Create a new entry (material, primitives)
                    self.materials.insert(key, vec![primitive_handle.id]);
                }

                // Get those nodes referring to this primitive
                if let Some(primitive_nodes) = self.primitives.get_mut(&primitive_handle.id) {
                    // Add this nodes to the list of nodes associated to this primitive if not already there
                    if !primitive_nodes.contains_key(&node_handle.id) {
                        primitive_nodes.insert(node_handle.id, temp_transform);
                    }
                } else {
                    // Create a new entry in the primitive resources
                    let mut primitive_nodes = HashMap::new();
                    primitive_nodes.insert(node_handle.id, temp_transform);
                    self.primitives.insert(primitive_handle.id, primitive_nodes);
                }
            }
        }

        // Check if current node has a directional light and set it for rendering
        if model
            .directional_lights
            .get(node.directional_light)
            .is_some()
        {
            self.directional_light = node_handle;
        }

        // Check if current node has a point light and add it to the current list
        if model.point_lights.get(node.point_light).is_some() {
            self.point_lights.push(node_handle);
        }

        // Here we check if the current node has a camera, just add it
        if model.cameras.get(node.camera).is_some() {
            self.cameras.push((node.camera, node_handle));
        }

        // And all its children recursively
        for &child in node.children.iter() {
            self.draw(model, child, &temp_transform);
        }
    }

    /// Renders a shadowmap. It should be called after drawing.
    pub fn render_shadow<D: DrawableOnto>(&mut self, model: &Model, target: &D) {
        self.shadow_map = target.get_depth_texture().unwrap().handle;

        let framebuffer = target.get_framebuffer();
        framebuffer.bind();
        unsafe {
            gl::Viewport(
                0,
                0,
                framebuffer.extent.width as _,
                framebuffer.extent.height as _,
            );

            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(0.6, 0.5, 1.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Draw only depth
        let draw_shadow_program = &self.custom_shaders[Shaders::DEPTH as usize];
        draw_shadow_program.bind();

        // Bind directional light as camera view
        let light_node = model.nodes.get(self.directional_light).unwrap();
        // Create orthographic camera but how big?
        let camera = Camera::orthographic(
            framebuffer.virtual_extent.width / 64,
            framebuffer.virtual_extent.height / 64,
        );
        draw_shadow_program.bind_camera(&camera, &light_node);
        // Keep track for next pass
        self.light_space = camera.proj * light_node.trs.get_view();

        // Draw the scene from the light point of view
        for (primitive_id, node_res) in self.primitives.iter() {
            let primitive = &model.primitives[*primitive_id];

            // Bind the primitive, bind the nodes using that primitive, draw the primitive.
            draw_shadow_program.bind_primitive(&primitive);
            for (node_id, transform) in node_res.iter() {
                let node = &model.nodes[*node_id];
                draw_shadow_program.bind_node(node, &transform);
                draw_shadow_program.draw(node, primitive);
            }
        }

        self.shaders.clear();
        self.point_lights.clear();
        self.cameras.clear();
        self.materials.clear();
        self.primitives.clear()
    }

    /// Renders depth from offscreen framebuffer to the screen
    pub fn blit_depth<D: DrawableOnto>(&mut self, source: &CustomFramebuffer, target: &D) {
        let depth_texture = source.depth_texture.as_ref().unwrap();

        let framebuffer = target.get_framebuffer();
        framebuffer.bind();
        unsafe {
            gl::Viewport(
                0,
                0,
                framebuffer.extent.width as _,
                framebuffer.extent.height as _,
            );
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Bind depth read shader
        self.read_depth_program.enable();

        // Bind extent
        if self.read_depth_program.loc.extent >= 0 {
            unsafe {
                gl::Uniform2f(
                    self.read_depth_program.loc.extent,
                    depth_texture.extent.width as f32,
                    depth_texture.extent.height as f32,
                );
            }
        }

        // Bind camera
        self.screen_camera
            .bind(&self.read_depth_program, &self.screen_node);

        // Bind texture
        depth_texture.bind();

        // Bind quad
        self.quad_primitive.bind();

        // Bind node
        self.quad_node
            .bind(&self.read_depth_program, &na::Matrix4::identity());

        // Draw
        self.quad_primitive.draw();
    }

    /// Renders colors from offscreen framebuffer to the screen
    pub fn blit_color<D: DrawableOnto>(&mut self, source: &CustomFramebuffer, target: &D) {
        let color_texture = &source.color_textures[0];

        let framebuffer = target.get_framebuffer();
        framebuffer.bind();

        unsafe {
            gl::Viewport(
                0,
                0,
                framebuffer.extent.width as _,
                framebuffer.extent.height as _,
            );
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Bind color read shader
        self.read_color_program.enable();

        // Bind extent
        if self.read_color_program.loc.extent >= 0 {
            unsafe {
                gl::Uniform2f(
                    self.read_color_program.loc.extent,
                    color_texture.extent.width as f32,
                    color_texture.extent.height as f32,
                );
            }
        }

        // Bind camera
        self.screen_camera
            .bind(&self.read_color_program, &self.screen_node);

        // Bind texture
        color_texture.bind();

        // Bind quad
        self.quad_primitive.bind();

        // Bind node
        self.quad_node
            .bind(&self.read_color_program, &na::Matrix4::identity());

        // Draw
        self.quad_primitive.draw();
    }

    /// This should be called after drawing everything to trigger the actual GL rendering.
    pub fn render_geometry<D: DrawableOnto>(&mut self, model: &Model, target: &D) {
        // Rendering should follow this approach
        // foreach prog in programs:
        //   bind(prog)
        //   foreach mat in p.materials:
        //     bind(mat)
        //     foreach prim in mat.primitives:
        //       bind(prim)
        //       foreach node in prim.nodes:
        //         bind(node) -> draw(prim)
        let framebuffer = target.get_framebuffer();
        framebuffer.bind();

        unsafe {
            gl::Viewport(
                0,
                0,
                framebuffer.extent.width as _,
                framebuffer.extent.height as _,
            );

            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(0.2, 0.3, 0.5, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Need to bind programs one at a time
        for (&shader_id, material_ids) in self.shaders.iter() {
            let shader = &self.custom_shaders[shader_id as usize];
            shader.bind();
            shader.bind_time(self.delta);
            shader.bind_extent(
                framebuffer.virtual_extent.width as f32,
                framebuffer.virtual_extent.height as f32,
            );
            if let Some(light_node) = model.nodes.get(self.directional_light) {
                let light = model
                    .directional_lights
                    .get(light_node.directional_light)
                    .unwrap();
                shader.bind_sun(&light.color, light_node, &self.light_space);
            }
            shader.bind_shadow(self.shadow_map);

            // Draw the scene from all the points of view
            for (camera_handle, camera_node_handle) in self.cameras.iter() {
                let camera = model.cameras.get(*camera_handle).unwrap();
                let camera_node = model.nodes.get(*camera_node_handle).unwrap();
                shader.bind_camera(camera, camera_node);

                // Need to bind materials for a group of primitives that use the same one
                for material_id in material_ids.iter() {
                    let primitive_ids = &self.materials[material_id];

                    let material = &model.materials[*material_id];
                    material.bind(&model.textures, &model.colors);

                    for primitive_id in primitive_ids.iter() {
                        let primitive = &model.primitives[*primitive_id];
                        assert!(primitive.material.is_some());

                        // Bind the primitive, bind the nodes using that primitive, draw the primitive.
                        shader.bind_primitive(primitive);
                        let node_res = &self.primitives[primitive_id];
                        for (&node_id, transform) in node_res.iter() {
                            let node = &model.nodes[node_id];
                            shader.bind_node(node, transform);

                            shader.draw(node, primitive);
                        }
                    }
                }
            }
        }

        if self.sky.enabled {
            let sky_shader = self.custom_shaders[Shaders::SKY as usize]
                .as_any()
                .downcast_ref()
                .unwrap();
            let (_, camera_node) = self.cameras[0];
            let camera_node = model.nodes.get(camera_node).unwrap();
            self.sky.draw(sky_shader as _, camera_node);
        }

        self.shaders.clear();
        self.point_lights.clear();
        self.cameras.clear();
        self.materials.clear();
        self.primitives.clear();
    }

    pub fn render_gui<D: DrawableOnto>(&mut self, ui: imgui::Ui, target: &D) {
        target.get_framebuffer().bind();

        let [width, height] = ui.io().display_size;
        let [scale_w, scale_h] = ui.io().display_framebuffer_scale;
        let fb_width = width * scale_w;
        let fb_height = height * scale_h;

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            // There is no glPolygonMode in GLES3.2
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Viewport(0, 0, fb_width as _, fb_height as _);
        }

        let data = ui.render();

        let matrix = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -(height as f32), 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        self.gui_res.program.enable();

        unsafe {
            gl::UniformMatrix4fv(
                self.gui_res.program.loc.proj,
                1,
                gl::FALSE,
                matrix.as_ptr() as _,
            );
            gl::Uniform1i(self.gui_res.program.loc.tex_sampler, 0);
        }

        for draw_list in data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            self.gui_res.mesh_res.vao.bind();

            self.gui_res.mesh_res.vbo.bind();
            unsafe {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vtx_buffer.len() * std::mem::size_of::<imgui::DrawVert>()) as _,
                    vtx_buffer.as_ptr() as _,
                    gl::STREAM_DRAW,
                );
            }

            self.gui_res.mesh_res.ebo.bind();
            unsafe {
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (idx_buffer.len() * std::mem::size_of::<imgui::DrawIdx>()) as _,
                    idx_buffer.as_ptr() as _,
                    gl::STREAM_DRAW,
                );
            }

            for cmd in draw_list.commands() {
                match cmd {
                    imgui::DrawCmd::Elements {
                        count,
                        cmd_params:
                            imgui::DrawCmdParams {
                                clip_rect: [x, y, z, w],
                                texture_id,
                                idx_offset,
                                ..
                            },
                    } => {
                        unsafe {
                            gl::BindTexture(gl::TEXTURE_2D, texture_id.id() as _);
                            gl::Scissor(
                                (x * scale_w) as gl::types::GLint,
                                (fb_height - w * scale_h) as gl::types::GLint,
                                ((z - x) * scale_w) as gl::types::GLint,
                                ((w - y) * scale_h) as gl::types::GLint,
                            );
                        }

                        let idx_size = if std::mem::size_of::<imgui::DrawIdx>() == 2 {
                            gl::UNSIGNED_SHORT
                        } else {
                            gl::UNSIGNED_INT
                        };

                        unsafe {
                            gl::DrawElements(
                                gl::TRIANGLES,
                                count as _,
                                idx_size,
                                (idx_offset * std::mem::size_of::<imgui::DrawIdx>()) as _,
                            );
                        }
                    }
                    imgui::DrawCmd::ResetRenderState => {
                        unimplemented!("DrawCmd::ResetRenderState not implemented");
                    }
                    imgui::DrawCmd::RawCallback { .. } => {
                        unimplemented!("User callbacks not implemented");
                    }
                }
            }
        }
    }
}
