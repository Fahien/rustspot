// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let width = 480;
    let height = 320;
    let mut spot = Spot::builder().width(width).height(height).build();

    let (mut model, root) = create_model(spot.gfx.video.profile);

    // Create a texture framebuffer half the size of the window
    spot.gfx.default_framebuffer.virtual_extent = Extent2D::new(width / 2, height / 2);
    let offscreen_extent = spot.gfx.default_framebuffer.virtual_extent;
    let depth_texture = Texture::depth(offscreen_extent);
    let framebuffer = Framebuffer::builder()
        .extent(offscreen_extent)
        .depth_attachment(&depth_texture)
        .build();
    let depth_texture = model.textures.push(depth_texture);

    // Create blur shader
    let blur_shader = model.programs.push(ShaderProgram::open(
        spot.gfx.video.profile,
        "res/shader/vert.glsl",
        "res/shader/depth.frag.glsl",
    ));

    // Create quad
    let mut screen_material = Material::new(depth_texture);
    screen_material.shader = blur_shader;
    let screen_material = model.materials.push(screen_material);
    let screen_quad = model.primitives.push(Primitive::quad(screen_material));
    let screen_mesh = model.meshes.push(Mesh::new(vec![screen_quad]));
    let mut quad_node = Node::new();
    quad_node.trs.scale(1.0, 1.0, 1.0);
    quad_node.mesh = screen_mesh;
    let quad_node = model.nodes.push(quad_node);

    let screen_camera = model.cameras.push(Camera::orthographic(1, 1));
    let mut camera_node = Node::new();
    camera_node.trs.translate(0.0, 0.0, 1.0);
    camera_node.camera = screen_camera;
    let camera_node = model.nodes.push(camera_node);

    let mut screen_node = Node::new();
    screen_node.children.push(camera_node);
    screen_node.children.push(quad_node);
    let screen_node = model.nodes.push(screen_node);

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                _ => println!("{:?}", event),
            }
        }

        let delta = spot.update();

        let rot =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 2.0);
        model.nodes.get_mut(&root).unwrap().trs.rotate(&rot);

        // Render something

        framebuffer.bind();

        unsafe {
            gl::Viewport(
                0,
                0,
                offscreen_extent.width as _,
                offscreen_extent.height as _,
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

        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());
        spot.gfx.renderer.present(&framebuffer, &model);

        // Render GUI
        let ui = spot.gfx.gui.frame();

        // Draw gui here before drawing it
        imgui::Window::new(imgui::im_str!("RustSpot"))
            .position([60.0, 60.0], imgui::Condition::FirstUseEver)
            .size([300.0, 60.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text("Hello world!");
            });

        spot.gfx.default_framebuffer.bind();
        unsafe {
            gl::Viewport(
                0,
                0,
                spot.gfx.video.extent.width as _,
                spot.gfx.video.extent.height as _,
            );
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Draw a simple triangle which cover the whole screen
        spot.gfx
            .renderer
            .draw(&model, &screen_node, &na::Matrix4::identity());
        spot.gfx
            .renderer
            .present(&spot.gfx.default_framebuffer, &model);

        spot.gfx.renderer.draw_gui(ui);

        // Present to the screen
        spot.gfx.swap_buffers();
    }
}

fn create_model(profile: sdl2::video::GLProfile) -> (Model, Handle<Node>) {
    let mut model = Model::new();

    // Shaders
    model.programs.push(ShaderProgram::open(
        profile,
        "res/shader/vert.glsl",
        "res/shader/frag.glsl",
    ));

    let texture = model.textures.push(Texture::open("res/img/lena.png"));

    // Create a material with the previous texture
    let material = model.materials.push(Material::new(texture));

    // Create a primitive quad with the previous material
    let primitive = model.primitives.push(Primitive::quad(material));

    // Create a mesh with a primitive quad
    let mut mesh = Mesh::new(vec![primitive]);
    mesh.name = String::from("quad");
    let mesh = model.meshes.push(mesh);

    let mut root = Node::new();
    root.name = String::from("root");

    let camera = model.cameras.push(Camera::perspective());

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node.camera = camera;
    camera_node.trs.translate(0.0, 0.0, 1.8);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    let mut quad = Node::new();
    quad.name = String::from("quad node");
    quad.mesh = mesh;
    root.children.push(model.nodes.push(quad));

    let root = model.nodes.push(root);

    (model, root)
}
