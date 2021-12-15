use std::collections::HashSet;
use std::error::Error;
use std::ops::Range;

use glsl::parser::Parse;
use glsl::syntax::{Declaration, ExternalDeclaration};

use super::*;

fn get_uniforms(code: &str) -> Vec<String> {
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

/// Returns a set with the name of the uniforms of both vertex and fragment shader
fn get_all_uniforms(code: &ShaderCode) -> HashSet<String> {
    let mut uniforms: HashSet<String> = HashSet::new();
    uniforms.extend(get_uniforms(&code.vert));
    uniforms.extend(get_uniforms(&code.frag));
    uniforms
}

pub fn generate(info: &ShaderInfo) -> Result<String, Box<dyn Error>> {
    let mut generated_code = String::new();

    for include in &info.includes {
        // Create enums for the includes
        generated_code.push_str(&format!(
            r#"
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum {0}{1}Variant {{
"#,
            info.camelcase, include.camelcase
        ));

        for variant in &include.variants {
            generated_code.push_str(&format!("    {},\n", variant.name));
        }

        generated_code.push_str("}\n");

        generated_code.push_str(&format!(
            r#"
impl {0}{1}Variant {{
    pub fn all() -> Vec<{0}{1}Variant> {{
        vec![
"#,
            info.camelcase, include.camelcase
        ));

        for variant in &include.variants {
            generated_code.push_str(&format!(
                "            {}{}Variant::{},\n",
                info.camelcase, include.camelcase, variant.name
            ));
        }

        generated_code.push_str(
            r#"        ]
    }
"#,
        );

        // As string slice
        generated_code.push_str(
            r#"
    pub fn as_str(&self) -> &str {
        match self {
"#,
        );

        for variant in &include.variants {
            generated_code.push_str(&format!(
                "            {0}{1}Variant::{2} => \"{2}\",\n",
                info.camelcase, include.camelcase, variant.name
            ));
        }

        generated_code.push_str(
            r#"        }
    }
"#,
        );

        generated_code.push_str("}\n");
    }

    for variant in &info.variants {
        let variant_code = generate_variant(info, variant)?;
        generated_code.push_str(&variant_code);
    }

    // More than one variants means we need to create a
    // parent shader with a multidimensional array of variants
    if info.variants.len() > 1 {
        let parent_code = generate_parent(info)?;
        generated_code.push_str(&parent_code);
    }

    Ok(generated_code)
}

fn generate_array(
    generated_code: &mut String,
    variant_index: &mut usize,
    info: &ShaderInfo,
    include: &Include,
    next_includes: &[Include],
    indent: String,
) {
    generated_code.push_str(&format!("{}[\n", indent));

    if next_includes.is_empty() {
        for _ in &include.variants {
            generated_code.push_str(&format!(
                "        {}Shaders::{},\n",
                indent, info.variants[*variant_index].camelcase
            ));
            *variant_index += 1;
        }
    } else {
        for _ in &include.variants {
            generate_array(
                generated_code,
                variant_index,
                info,
                &next_includes[0],
                &next_includes[1..],
                format!("{}    ", indent),
            );
        }
    }

    generated_code.push_str(&format!("{}],\n", indent));
}

fn generate_parent(info: &ShaderInfo) -> Result<String, Box<dyn Error>> {
    let mut generated_code = String::new();

    generated_code.push_str(&format!(
        "\npub const {}_VARIANTS: ",
        info.prefix.to_uppercase()
    ));

    // Define multidimensional array
    for _ in &info.includes {
        generated_code.push_str("[");
    }
    generated_code.push_str("Shaders");
    for include in &info.includes {
        generated_code.push_str(&format!(";{}]", include.variants.len()));
    }
    generated_code.push_str(" = ");

    // Populate multidimensional array
    let mut variant_index = 0;
    // TODO fix indentation probably
    generate_array(
        &mut generated_code,
        &mut variant_index,
        &info,
        &info.includes[0],
        &info.includes[1..],
        "".to_string(),
    );

    generated_code.replace_range(generated_code.len() - 2.., ";");

    generated_code.push_str("\n");

    Ok(generated_code)
}

fn generate_variant(info: &ShaderInfo, variant: &VariantInfo) -> Result<String, Box<dyn Error>> {
    let mut generated_code = String::new();

    let vs_path_string = variant
        .get_gen_path(&info.dir, util::VERT_SUFFIX)
        .to_string_lossy()
        .to_string()
        .replace("\\", "/");
    let fs_path_string = variant
        .get_gen_path(&info.dir, util::FRAG_SUFFIX)
        .to_string_lossy()
        .to_string()
        .replace("\\", "/");

    // These are useful to create the location structure code
    let uniform_strings = get_all_uniforms(&variant.code);

    generated_code.push_str(&format!("\npub struct {}Loc {{\n", variant.camelcase));

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
        variant.camelcase
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
        let vert_src = include_bytes!("../../{1}");
        let frag_src = include_bytes!("../../{2}");

        let vs = Shader::new(gl::VERTEX_SHADER, vert_src)
            .expect("Failed to create shader from {1}");
        let fs = Shader::new(gl::FRAGMENT_SHADER, frag_src)
            .expect("Failed to create shader from {2}");
        let program = ShaderProgram::new(vs, fs);
        let loc = {0}Loc::new(&program);
        Self {{
            program, loc
        }}
    }}
"#,
        variant.camelcase,
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
        variant.camelcase
    ));

    // Associate texture units and samplers
    if uniform_strings.contains("tex_sampler") {
        generated_code.push_str("        unsafe { gl::Uniform1i(self.loc.tex_sampler, 0) };\n");
    }
    if uniform_strings.contains("normal_sampler") {
        generated_code.push_str("        unsafe { gl::Uniform1i(self.loc.normal_sampler, 2) };\n");
    }
    if uniform_strings.contains("occlusion_sampler") {
        generated_code
            .push_str("        unsafe { gl::Uniform1i(self.loc.occlusion_sampler, 3) };\n");
    }
    if uniform_strings.contains("mr_sampler") {
        generated_code.push_str("        unsafe { gl::Uniform1i(self.loc.mr_sampler, 4) };\n");
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
"#);

        // Normal sampler
        if uniform_strings.contains("normal_sampler") {
            generated_code.push_str(
                r#"
        // Bind normal map
        if let Some(normals_handle) = material.normals {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 2);
                textures.get(normals_handle).unwrap().bind();
                gl::ActiveTexture(gl::TEXTURE0);
            }
        }
"#,
            );
        }
        // Occlusion sampler
        if uniform_strings.contains("occlusion_sampler") {
            generated_code.push_str(
                r#"
        // Bind metallic roughness texture
        if let Some(occlusion_handle) = material.occlusion {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 3);
                textures.get(occlusion_handle).unwrap().bind();
                gl::ActiveTexture(gl::TEXTURE0);
            }
        }
"#,
            );
        }

        if uniform_strings.contains("mr_sampler") {
            generated_code.push_str(
                r#"
        // Bind metallic roughness texture
        if let Some(mr_handle) = material.metallic_roughness {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 4);
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
            generated_code.push_str(
                r#"
        // Instance count
        let instance_count = std::cmp::max(1, node.transforms.len());
        if self.loc.instance_count >= 0 {
            unsafe {
                gl::Uniform1i(
                    self.loc.instance_count,
                    instance_count as _,
                );
            }
        }

        // Transforms array
        let transform_ptr = if node.transforms.len() > 0 {
            node.transforms.as_ptr() as _
        } else {
            transform.as_ptr() as _
        };
        unsafe {
            gl::UniformMatrix4fv(self.loc.models, instance_count as _, gl::FALSE, transform_ptr);
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
                    primitive.index_type,
                    0 as _,
                );
            }
        } else {
            let mut remaining_instance_count = instance_count;
            let draw_calls = ((instance_count - 1) / 128) + 1;
            for i in 0..draw_calls {
                let batch_count = std::cmp::min(remaining_instance_count, 128);
                remaining_instance_count -= batch_count;

                // Instance count
                if self.loc.instance_count >= 0 {
                    unsafe {
                        gl::Uniform1i(
                            self.loc.instance_count,
                            instance_count as _,
                        );
                    }
                }

                // Transforms array
                if node.transforms.len() > 0 {
                    unsafe {
                        gl::UniformMatrix4fv(self.loc.models, batch_count as _, gl::FALSE, node.transforms[i * 128].as_ptr() as _);
                    }
                }

                unsafe {
                    gl::DrawElementsInstanced(
                        gl::TRIANGLES,
                        primitive.indices.len() as _,
                        primitive.index_type,
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
                    primitive.index_type,
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
