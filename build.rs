// build.rs

use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, ReadDir};
use std::path::Path;

use glsl::parser::Parse;
use glsl::syntax::{Declaration, ExternalDeclaration};

const VERT_SUFFIX: &str = "vert.glsl";
const FRAG_SUFFIX: &str = "frag.glsl";

const HEADER: &str = r#"// Generated code, do not modify.
use crate::*;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::any::Any;
use std::collections::HashMap;

use nalgebra as na;

"#;

fn get_shader_prefixes(dir: ReadDir) -> Vec<String> {
    let mut shader_prefixes = vec![];
    for shader_name in dir
        .filter_map(|e| e.ok())
        .filter_map(|e| Some(e.file_name().to_string_lossy().to_string()))
        .filter(|e| e.ends_with(VERT_SUFFIX))
    {
        let shader_prefix_len = shader_name.len() - VERT_SUFFIX.len();
        let shader_prefix = &shader_name[0..shader_prefix_len];
        shader_prefixes.push(shader_prefix.to_string());
    }
    shader_prefixes
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut code = String::from(HEADER);

    let shaders_path = Path::new("res/shader");
    let shaders_dir = std::fs::read_dir(shaders_path)?;
    let shader_prefixes = get_shader_prefixes(shaders_dir);

    code.push_str(&generate_enum(&shader_prefixes)?);
    code.push_str(&generate_create_shaders(&shader_prefixes)?);

    for shader_prefix in &shader_prefixes {
        code.push_str(&generate(shaders_path, shader_prefix)?);
    }

    let dest_path = Path::new("src/rustspot/shaders.rs");
    fs::write(dest_path, code)?;

    println!("cargo:rerun-if-changed=res/shader;build.rs");
    Ok(())
}

fn _generate_path(shaders_path: &Path, shader_prefix: &str) -> Result<String, Box<dyn Error>> {
    let vs_path = shaders_path.join(shader_prefix).join(VERT_SUFFIX);
    Ok(std::format!("{}\n", vs_path.to_str().unwrap()))
}

fn to_camelcase(name: &str) -> String {
    let (symbol_indices, _): (Vec<usize>, Vec<char>) = name
        .chars()
        .enumerate()
        .filter(|(_, c)| *c == '-' || *c == '_')
        .unzip();

    let mut name = name.to_string();
    name.replace_range(0..1, &name[0..1].to_uppercase());

    for i in symbol_indices {
        if i < name.len() - 1 {
            let char = name.chars().nth(i + 1).unwrap().to_uppercase().to_string();
            name.replace_range(i + 1..i + 2, &char);
        }
    }

    let name = name.chars().filter(|&c| c != '_' && c != '-').collect();

    name
}

fn generate_enum(shader_prefixes: &Vec<String>) -> Result<String, Box<dyn Error>> {
    let mut code =
        String::from("#[derive(Hash, Eq, PartialEq, Copy, Clone)]\npub enum Shaders {\n");

    for shader_prefix in shader_prefixes {
        let shader_camel = to_camelcase(shader_prefix);
        code.push_str(&std::format!("    {},\n", shader_camel.to_uppercase(),));
    }

    code.push_str("}\n\n");

    Ok(code)
}

fn generate_create_shaders(shader_prefixes: &Vec<String>) -> Result<String, Box<dyn Error>> {
    let mut code =
        String::from("pub fn create_shaders() -> Vec<Box<dyn CustomShader>> {\n    vec![\n");

    for shader_prefix in shader_prefixes {
        let shader_camel = to_camelcase(shader_prefix);
        code.push_str(&std::format!(
            "        Box::new({}Shader::new()),\n",
            shader_camel
        ));
    }

    code.push_str("    ]\n}\n\n");

    Ok(code)
}

fn get_uniforms(code: &String) -> Vec<String> {
    let mut uniform_strings = vec![];
    let unit = glsl::syntax::ShaderStage::parse(&code).unwrap();

    for dec in &unit {
        match dec {
            ExternalDeclaration::Declaration(decl) => {
                if let Declaration::InitDeclaratorList(idl) = decl {
                    if let Some(id) = idl.head.name.as_ref() {
                        uniform_strings.push(String::from(id.as_str()));
                    }
                }
            }
            _ => (),
            //ExternalDeclaration::Preprocessor(_) => (),
            //ExternalDeclaration::FunctionDefinition(_) => (),
        };
    }

    uniform_strings
}

fn generate(shaders_path: &Path, shader_prefix: &str) -> Result<String, Box<dyn Error>> {
    let vs_path = shaders_path.join(std::format!("{}{}", shader_prefix, VERT_SUFFIX));
    let fs_path = shaders_path.join(std::format!("{}{}", shader_prefix, FRAG_SUFFIX));

    let vs_path_string = vs_path.to_string_lossy().to_string().replace("\\", "/");
    let fs_path_string = fs_path.to_string_lossy().to_string().replace("\\", "/");

    let vs_code = std::fs::read_to_string(&vs_path)?;
    let fs_code = std::fs::read_to_string(&fs_path)?;

    let mut uniform_strings: HashSet<String> = HashSet::new();
    uniform_strings.extend(get_uniforms(&vs_code));
    uniform_strings.extend(get_uniforms(&fs_code));

    let shader_camel = to_camelcase(shader_prefix);

    let mut generated_code = std::format!("\npub struct {}Loc {{\n", shader_camel);

    for uniform in &uniform_strings {
        generated_code.push_str(&std::format!("    pub {}: i32,\n", uniform));
    }

    generated_code.push_str(&std::format!(
        r#"}}

pub struct {0}Shader {{
    program: ShaderProgram,
    pub loc: {0}Loc,
}}

impl {0}Loc {{
    pub fn new(program: &ShaderProgram) -> Self {{
        {0}Loc {{
"#,
        shader_camel
    ));

    for uniform in &uniform_strings {
        generated_code.push_str(&std::format!(
            "            {}: program.get_uniform_location(\"{}\"),\n",
            uniform,
            uniform
        ));
    }

    generated_code.push_str(&std::format!(
        r#"        }}
    }}
}}

impl {}Shader {{
    pub fn new() -> Self {{
        let vert_path = Path::new("{1}");
        let frag_path = Path::new("{2}");

        let mut vert_src = Vec::<u8>::new();
        let mut frag_src = Vec::<u8>::new();

        File::open(vert_path)
            .expect("Failed to open vertex file")
            .read_to_end(&mut vert_src)
            .expect("Failed reading vertex file");
        File::open(frag_path)
            .expect("Failed to open fragment file")
            .read_to_end(&mut frag_src)
            .expect("Failed reading fragment file");

        let vs = Shader::new(gl::VERTEX_SHADER, &vert_src)
            .expect("Failed to create shader from {1}");
        let fs = Shader::new(gl::FRAGMENT_SHADER, &frag_src)
            .expect("Failed to create shader from {2}");
        let program = ShaderProgram::new(vs, fs);
        let loc = {0}Loc::new(&program);
        Self {{
            program, loc
        }}
    }}
"#,
        shader_camel,
        vs_path_string,
        fs_path_string
    ));

    generated_code.push_str(&std::format!(
        r#"}}

impl CustomShader for {}Shader {{
    fn as_any(&self) -> &dyn Any {{
        self
    }}

    fn bind(&self) {{
        self.program.enable();
"#,
        shader_camel
    ));

    // Associate texture units and samplers
    if uniform_strings.contains("tex_sampler") {
        generated_code.push_str("        unsafe { gl::Uniform1i(self.loc.tex_sampler, 0) };\n");
    }
    if uniform_strings.contains("normal_sampler") {
        generated_code.push_str("        unsafe { gl::Uniform1i(self.loc.normal_sampler, 2) };\n");
    }

    generated_code.push_str("    }\n");

    if uniform_strings.contains("time") {
        generated_code.push_str(
            r#"
    fn bind_time(&self, delta: f32) {
        unsafe {
            gl::Uniform1f(self.loc.time, delta);
        }
    }
"#,
        );
    }

    if uniform_strings.contains("extent") {
        generated_code.push_str(
            r#"
    fn bind_extent(&self, with: f32, height: f32) {
        unsafe {
            gl::Uniform2f(
                self.loc.extent,
                width,
                height,
            );
        }
    }
"#,
        );
    }

    if uniform_strings.contains("light_color") {
        generated_code.push_str(
            r#"
    fn bind_sun(&self, light_color: &[f32; 3], light_node: &Node, light_space: &na::Matrix4<f32>) {
        // Light direction should point towards light source thus we negate it
        let direction = -light_node.trs.get_forward();

        unsafe {
            gl::Uniform3fv(self.loc.light_color, 1, light_color as _);
            gl::Uniform3fv(self.loc.light_direction, 1, direction.as_ptr() as _);
            gl::UniformMatrix4fv(
                self.loc.light_space,
                1,
                gl::FALSE,
                light_space.as_ptr(),
            );
        }
    }
"#,
        );
    }

    if uniform_strings.contains("shadow_sampler") {
        generated_code.push_str(
            r#"
    fn bind_shadow(&self, shadow_map: u32) {
        unsafe {
            gl::Uniform1i(self.loc.shadow_sampler, 1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, shadow_map);
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }
"#,
        );
    }

    if uniform_strings.contains("view") {
        generated_code.push_str(
            r#"
    fn bind_camera(&self, camera: &Camera, node: &Node) {
        let view = node.trs.get_view();
        unsafe {
            gl::UniformMatrix4fv(self.loc.view, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.loc.proj, 1, gl::FALSE, camera.proj.as_ptr());
"#,
        );

        if uniform_strings.contains("cam_pos") {
            generated_code.push_str(
                r#"
            let pos = node.trs.get_translation();
            gl::Uniform3fv(self.loc.cam_pos, 1, pos.as_ptr());
"#,
            );
        }

        if uniform_strings.contains("billboard") {
            generated_code.push_str(
                r#"
            let mut cam_pos = node.trs.get_translation();
            cam_pos.y = 0.0;
            let up = na::Vector3::y();
            let billboard = na::Rotation3::face_towards(&cam_pos, &up).to_homogeneous().remove_column(3).remove_row(3);
            gl::UniformMatrix3fv(self.loc.billboard, 1, gl::FALSE, billboard.as_ptr());
"#
            )
        }

        generated_code.push_str("        }\n    }\n");
    }

    // Bind material
    if uniform_strings.contains("tex_sampler") {
        generated_code.push_str(r#"
    fn bind_material(&self, textures: &Pack<Texture>, colors: &HashMap<Color, Texture>, material: &Material) {
        // Bind albedo map
        if let Some(texture_handle) = material.texture {
            textures.get(texture_handle).unwrap().bind();
        } else {
            colors.get(&material.color).unwrap().bind();
        }

        // Bind normal map
        if let Some(normals_handle) = material.normals {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 2);
                textures.get(normals_handle).unwrap().bind();
                gl::ActiveTexture(gl::TEXTURE0);
            }
        }
"#);

        if uniform_strings.contains("mr_sampler") {
            generated_code.push_str(
                r#"
        // Bind metallic roughness texture
        if let Some(mr_handle) = material.metallic_roughness {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 3);
                textures.get(mr_handle).unwrap().bind();
                gl::ActiveTexture(gl::TEXTURE0);
            }
        }
"#,
            );
        }

        if uniform_strings.contains("metallic") {
            generated_code.push_str(
                "\n        unsafe { gl::Uniform1f(self.loc.metallic, material.metallic); }\n",
            );
        }

        if uniform_strings.contains("metallic") {
            generated_code.push_str(
                "        unsafe { gl::Uniform1f(self.loc.roughness, material.roughness); }\n",
            );
        }

        generated_code.push_str("    }\n");
    }

    // Bind node
    if uniform_strings.contains("model") {
        generated_code.push_str(
            r#"
    fn bind_node(&self, node: &Node, transform: &na::Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.loc.model, 1, gl::FALSE, transform.as_ptr());
        }
"#,
        );

        if uniform_strings.contains("models") {
            generated_code.push_str(r#"
            let instance_count = std::cmp::max(1, node.transforms.len());
            unsafe {
                gl::Uniform1i(
                    self.loc.instance_count,
                    instance_count as _,
                );
                gl::UniformMatrix4fv(self.loc.models, instance_count as _, gl::FALSE, node.transforms.as_ptr() as _);
            }
"#,
    );
        }

        if uniform_strings.contains("model_intr") {
            generated_code.push_str(
                r#"
        if let Some(intr) = transform
            .remove_column(3)
            .remove_row(3)
            .try_inverse() {
            unsafe {
                gl::UniformMatrix3fv(self.loc.model_intr, 1, gl::TRUE, intr.as_ptr());
            }
        } else {
            panic!();
        }
"#,
            );
        }

        if uniform_strings.contains("node_id") {
            generated_code.push_str(
                r#"
        unsafe {
            gl::Uniform1i(self.loc.node_id, node.id as i32);
        }
"#,
            );
        }

        generated_code.push_str("    }\n");
    }

    generated_code.push_str(
        r#"
    fn bind_primitive(&self, primitive: &Primitive) {
        primitive.bind();
    }
"#,
    );

    if uniform_strings.contains("instance_count") {
        // Draw method
        generated_code.push_str(r#"
    fn draw(&self, node: &Node, primitive: &Primitive) {
        let instance_count = std::cmp::max(1, node.transforms.len());

        if instance_count == 0 {
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    primitive.indices.len() as _,
                    gl::UNSIGNED_SHORT,
                    0 as _,
                );
            }
        } else {
            let mut remaining_instance_count = instance_count;
            let draw_calls = ((instance_count - 1) / 128) + 1;
            for i in 0..draw_calls {
                let batch_count = std::cmp::min(remaining_instance_count, 128);
                remaining_instance_count -= batch_count;
                unsafe {
                    gl::Uniform1i(
                        self.loc.instance_count,
                        instance_count as _,
                    );

                    gl::UniformMatrix4fv(self.loc.models, batch_count as _, gl::FALSE, node.transforms[i * 128].as_ptr() as _);

                    gl::DrawElementsInstanced(
                        gl::TRIANGLES,
                        primitive.indices.len() as _,
                        gl::UNSIGNED_SHORT,
                        0 as _,
                        batch_count as _,
                    );
                }
            }
        }
    }
}
"#);
    } else {
        generated_code.push_str(
            r#"
    fn draw(&self, node: &Node, primitive: &Primitive) {
        if primitive.indices.len() == 0 {
            unsafe {
                gl::DrawArrays(
                    gl::TRIANGLES,
                    0,
                    primitive.vertices.len() as _,
                )
            }
        } else {
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    primitive.indices.len() as _,
                    gl::UNSIGNED_SHORT,
                    0 as _,
                );
            }
        }
    }
}
"#,
        );
    }

    Ok(generated_code)
}
