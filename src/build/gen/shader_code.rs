use std::path::{Path, PathBuf};

use super::*;

#[derive(Clone)]
pub struct IncludeVariant {
    /// Name of the include, such as "Occlusion"
    pub prefix: String,

    /// Name of the variant, such as "Default"
    pub name: String,

    pub path: PathBuf,
    pub index: usize,
}

impl IncludeVariant {
    pub fn new(prefix: String, name: String, path: PathBuf, index: usize) -> Self {
        let name = util::to_camelcase(&name);
        Self {
            prefix,
            name,
            path,
            index,
        }
    }
}

/// Returns a list of include variants
/// This function looks in the directory for any file starting with `include_name`
/// and generate an include variants for each file.
/// - `include_name` is the snake case name of the include
/// - `dir` is the directory of the include file
fn find_variants(include_name: &str, dir: &Path) -> Vec<IncludeVariant> {
    let default_file_name = format!("{}.glsl", include_name);
    let default_path = dir.join(&default_file_name);
    let mut variants = vec![IncludeVariant::new(
        include_name.to_string(),
        "Default".to_string(),
        default_path,
        0,
    )];

    // Get all other include files starting with this include_name
    let include_dir = std::fs::read_dir(dir).expect("Failed to read variant dir");
    for (index, variant_file_name) in include_dir
        .filter_map(|x| {
            if let Ok(entry) = x {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let file_name = entry.file_name();
                        let file_name = file_name.to_string_lossy();
                        if file_name != default_file_name && file_name.starts_with(include_name) {
                            return Some(file_name.to_string());
                        }
                    }
                }
            }
            // Filter this file
            None
        })
        .enumerate()
    {
        let last_dot_rev_index = variant_file_name
            .chars()
            .rev()
            .enumerate()
            .position(|(_, c)| c == '.')
            .expect("Failed to find last dot");
        let last_dot_index = variant_file_name.len() - last_dot_rev_index;
        let variant_name = variant_file_name[include_name.len() + 1..last_dot_index - 1].to_string();
        let variant_path = dir.join(&variant_file_name);
        variants.push(IncludeVariant::new(
            include_name.to_string(),
            variant_name,
            variant_path,
            index,
        ));
    }

    variants
}

pub struct Include {
    /// Line number where this include was found in the original shader
    pub index: usize,

    /// Snake case name of the include, such as "occlusion"
    pub name: String,

    /// Camel case name of the include, such as "Occlusion"
    pub camelcase: String,

    pub variants: Vec<IncludeVariant>,
}

impl Include {
    pub fn new(index: usize, name: String, dir: &Path) -> Self {
        let camelcase = util::to_camelcase(&name);
        let variants = find_variants(&name, dir);
        Self {
            index,
            name,
            camelcase,
            variants,
        }
    }
}

// Returns the list of names of the includes in the code
pub fn get_includes(code: &str, dir: &Path) -> Vec<Include> {
    let includes = code
        .lines()
        .enumerate()
        // Find lines starting with #include
        .filter(|(_, line)| line.starts_with("#include"))
        .map(|(index, line)| {
            // Get the string with "" or <>
            let include = line.splitn(3, "\"").nth(1).unwrap_or_else(|| {
                line.split("<")
                    .nth(1)
                    .expect("Failed to get include name")
                    .split(">")
                    .nth(0)
                    .expect("Failed to get include name")
            });
            // Get the name without the extension
            (
                index,
                include
                    .split('.')
                    .nth(0)
                    .expect("Failed to get include name")
                    .to_string(),
            )
        });

    includes
        .map(|(index, name)| Include::new(index, name, dir))
        .collect()
}

/// Returns the code with this include variant resolved
pub fn resolve_includes(
    mut code: ShaderCode,
    include: &Include,
    include_variant: &IncludeVariant,
) -> ShaderCode {
    let include_code = std::fs::read_to_string(&include_variant.path).expect(&format!(
        "Failed to include {}",
        include_variant.path.to_string_lossy()
    ));
    // Insert string within code substituting include line
    let mut lines = code.frag.lines();
    let include_index = lines
        .position(|line| line.starts_with("#include"))
        .expect("Failed to find include line to resolve");
    // TODO solve vert as well?
    let mut lines: Vec<String> = code.frag.lines().map(|l| String::from(l)).collect();

    eprintln!("include:{}", include.index);
    lines[include_index] = include_code;

    code.frag = lines.join("\n");
    code
}
