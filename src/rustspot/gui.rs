use crate::*;

pub struct GuiRes {
    _font_texture: Texture,
    pub program: ShaderProgram,
    pub mesh_res: MeshRes,
}

impl GuiRes {
    pub fn new(fonts: &mut imgui::FontAtlasRefMut) -> Self {
        // Font
        let texture = fonts.build_rgba32_texture();
        let font_texture = Texture::builder()
            .extent(Extent2D::new(texture.width, texture.height))
            .data(texture.data)
            .build()
            .unwrap();
        fonts.tex_id = (font_texture.handle as usize).into();

        // Shaders
        let vert_source = r#"
        layout (location = 0) in vec2 in_position;
        layout (location = 1) in vec2 in_tex_coords;
        layout (location = 2) in vec4 in_color;

        uniform mat4 proj;

        out vec2 tex_coords;
        out vec4 color;

        void main()
        {
            tex_coords = in_tex_coords;
            color = in_color;
            gl_Position = proj * vec4(in_position, 0.0, 1.0);
        }
        "#;

        let frag_source = r#"
        precision mediump float;

        in vec2 tex_coords;
        in vec4 color;

        uniform sampler2D tex_sampler;

        out vec4 out_color;

        void main()
        {
            vec4 alpha = vec4(1.0, 1.0, 1.0, 0.75);
            out_color = alpha * color * texture(tex_sampler, tex_coords);
        }
        "#;

        let vert = Shader::new(gl::VERTEX_SHADER, vert_source.as_bytes())
            .expect("Failed to create imgui vertex shader");
        let frag = Shader::new(gl::FRAGMENT_SHADER, frag_source.as_bytes())
            .expect("Failed to create imgui fragment shader");

        let program = ShaderProgram::new(vert, frag);

        // Mesh resources
        let mesh_res = MeshRes::new();

        mesh_res.vao.bind();
        mesh_res.vbo.bind();
        mesh_res.ebo.bind();

        let stride = std::mem::size_of::<imgui::DrawVert>() as i32;

        unsafe {
            // Position
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0 as _);
            gl::EnableVertexAttribArray(0);

            // Texture coordinates
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (2 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(1);

            // Color
            gl::VertexAttribPointer(
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                stride,
                (4 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(2);
        }

        Self {
            _font_texture: font_texture,
            program,
            mesh_res,
        }
    }
}
