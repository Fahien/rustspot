use std::error::Error;

use super::*;

fn get_enum_values(shader_infos: &[ShaderInfo]) -> Vec<&str> {
    let mut values = vec![];

    for info in shader_infos {
        if info.variants.is_empty() {
            values.push(info.camelcase.as_str());
        } else {
            for variant in &info.variants {
                values.push(variant.camelcase.as_str());
            }
        }
    }

    values
}

pub fn generate(shader_infos: &Vec<ShaderInfo>) -> Result<String, Box<dyn Error>> {
    let mut code =
        String::from("#[derive(Hash, Eq, PartialEq, Copy, Clone)]\npub enum Shaders {\n");

    let enum_values = get_enum_values(shader_infos);

    for value in &enum_values {
        code.push_str(&format!("    {},\n", value));
    }

    code.push_str("}\n\n");

    // Display enum
    code.push_str(
        r#"
impl fmt::Display for Shaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
"#,
    );

    // Next method for Shader enum
    code.push_str(
        r#"
impl Shaders {
    /// Get next shader in the enum
    pub fn next(&self) -> Option<Self> {
        match self {
"#,
    );

    for i in 0..enum_values.len() {
        let value = &enum_values[i];

        let next_upper = if i + 1 < enum_values.len() {
            let next_index = (i + 1) % enum_values.len();
            let next_info = &enum_values[next_index];
            format!("Some(Self::{})", next_info)
        } else {
            "None".to_string()
        };

        code.push_str(&std::format!(
            "            Self::{} => {},\n",
            value,
            next_upper
        ));
    }

    code.push_str(
        r#"        }
    }
"#,
    );

    code.push_str(
        r#"
    /// Get previous shader in the enum
    pub fn prev(&self) -> Option<Self> {
        match self {
"#,
    );

    for i in 0..enum_values.len() {
        let prev_upper = if i > 0 {
            let prev_value = &enum_values[i - 1];
            format!("Some(Self::{})", prev_value)
        } else {
            "None".to_string()
        };

        let value = &enum_values[i];

        code.push_str(&std::format!(
            "            Self::{} => {},\n",
            value,
            prev_upper
        ));
    }

    code.push_str(
        r#"        }
    }
"#,
    );

    // As string slice
    code.push_str(
        r#"
    /// Returns the name of this shader
    pub fn as_str(&self) -> &str {
        match self {
"#,
    );

    for info in &enum_values {
        code.push_str(&std::format!("            Self::{0} => \"{0}\",\n", info));
    }

    code.push_str(
        r#"        }
    }
"#,
    );

    // First shader
    code.push_str(&format!(
        r#"
    pub fn first() -> Self {{
        Self::{}
    }}
"#,
        enum_values[0]
    ));

    // Last shader
    code.push_str(&format!(
        r#"
    pub fn last() -> Self {{
        Self::{}
    }}
"#,
        enum_values.last().unwrap()
    ));

    // End enum impl
    code.push_str("}\n\n");

    Ok(code)
}
