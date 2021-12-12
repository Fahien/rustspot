use std::error::Error;

use super::*;

pub fn generate(shader_infos: &Vec<ShaderInfo>) -> Result<String, Box<dyn Error>> {
    let mut code =
        String::from("pub fn create_shaders() -> Vec<Box<dyn CustomShader>> {\n    vec![\n");

    for info in shader_infos {
        if info.variants.is_empty() {
            code.push_str(&format!(
                "        Box::new({}Shader::new()),\n",
                info.camelcase
            ));
        } else {
            for variant in &info.variants {
                code.push_str(&format!(
                    "        Box::new({}Shader::new()),\n",
                    variant.camelcase
                ));
            }
        }
    }

    code.push_str("    ]\n}\n");

    Ok(code)
}
