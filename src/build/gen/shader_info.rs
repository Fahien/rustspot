use std::{
    error::Error,
    path::{Path, PathBuf},
};

use super::*;

// Code of this shader with all its includes resolved
#[derive(Clone)]
pub struct ShaderCode {
    pub vert: String,
    pub frag: String,
}

impl ShaderCode {
    pub fn new(vert: String, frag: String) -> Self {
        Self { vert, frag }
    }
}

/// This is something that can be associated with an include file
pub struct VariantInfo {
    // Name of the shader used to generate a file
    pub name: String,

    pub camelcase: String,

    pub code: ShaderCode,
}

impl VariantInfo {
    pub fn new(name: String, code: ShaderCode) -> Self {
        let camelcase = util::to_camelcase(&name);
        Self {
            name,
            camelcase,
            code,
        }
    }

    pub fn get_gen_path(&self, dir: &Path, suffix: &str) -> PathBuf {
        dir.join(format!("gen/{}{}", self.name, suffix))
    }

    pub fn write_code(&self, dir: &Path, code: &str, suffix: &str) {
        // Create the gen folder if not already there
        let gen_dir = dir.join("gen");
        std::fs::create_dir(gen_dir).ok();

        let gen_path = self.get_gen_path(dir, suffix);
        let gen_path_str = gen_path.to_string_lossy().to_string();
        std::fs::write(gen_path, code).expect(&format!("Failed to generate {}", gen_path_str));
    }

    pub fn write(&self, dir: &Path) {
        self.write_code(dir, &self.code.vert, util::VERT_SUFFIX);
        self.write_code(dir, &self.code.frag, util::FRAG_SUFFIX);
    }
}

fn solve_variant(
    code: ShaderCode,
    include: &Include,
    include_variant: &IncludeVariant,
) -> ShaderCode {
    shader_code::resolve_includes(code, include, include_variant)
}

/// shader_name is something like "pbr"
/// solved is the shader code solved so far
/// includes is the list of includes still to solve
/// include_variants List of include variants solved so far
fn solve_includes(
    shader_name: &str,
    solved: ShaderCode,
    includes: &[Include],
    include_variants: Vec<IncludeVariant>,
) -> Vec<VariantInfo> {
    let mut variants = vec![];

    if includes.is_empty() {
        // No more includes to solve, generate shader variant
        let name = include_variants
            .iter()
            .map(|v| format!("{}-{}", v.prefix, v.name.to_lowercase()))
            .collect::<Vec<String>>()
            .join("-");
        let name = format!("{}-{}", shader_name, name);
        let variant = VariantInfo::new(name, solved);
        variants.push(variant);
    } else {
        let include = &includes[0];
        for include_variant in &include.variants {
            // Solve this include variant
            let next_solved = solve_variant(solved.clone(), include, include_variant);

            // Mark this variant as solved
            let mut include_variants = include_variants.clone();
            include_variants.push(include_variant.clone()); 

            // Solve next include
            let mut next_variants =
                solve_includes(shader_name, next_solved, &includes[1..], include_variants);
            variants.append(&mut next_variants);
        }
    }

    variants
}

/// shader_name is something like "pbr"
/// code is the unsolved code with include to solve
/// includes is the list of includes to solve
pub fn get_variants(shader_name: &str, code: ShaderCode, includes: &[Include]) -> Vec<VariantInfo> {
    if includes.is_empty() {
        vec![VariantInfo::new(shader_name.to_string(), code)]
    } else {
        solve_includes(shader_name, code, includes, vec![])
    }
}

pub struct ShaderInfo {
    pub prefix: String,
    pub camelcase: String,
    pub uppercase: String,
    pub dir: PathBuf,

    // A shader can have various includes, each one with various variants
    pub includes: Vec<Include>,

    pub variants: Vec<VariantInfo>,
}

impl ShaderInfo {
    pub fn new(prefix: String, dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        let camelcase = util::to_camelcase(&prefix);
        let uppercase = camelcase.to_uppercase();

        let vert_path = dir.join(&format!("{}{}", prefix, util::VERT_SUFFIX));
        let vert = std::fs::read_to_string(vert_path)?;
        let frag_path = dir.join(&format!("{}{}", prefix, util::FRAG_SUFFIX));
        let frag = std::fs::read_to_string(frag_path)?;

        let code = ShaderCode::new(vert, frag);

        // Collect includes from fragment shader
        let includes = get_includes(&code.frag, &dir);

        eprintln!("shader:{}:include_count:{}", prefix, includes.len());
        for include in &includes {
            for variant in &include.variants {
                eprintln!("include:{}:variant:{}", include.name, variant.name);
            }
        }

        let variants = get_variants(&prefix, code, &includes);
        eprintln!("shader:{}:variant_count:{}", prefix, variants.len());
        for variant in &variants {
            eprintln!("variant:{}:code:{}", variant.name, "");
            variant.write(&dir);
        }

        // TODO: should generate gen shaders for all possible variants
        //let (vert_code, vert_includes) = generate_gen_shader(&dir, &prefix, util::VERT_SUFFIX)?;
        //let (frag_code, frag_includes) = generate_gen_shader(&dir, &prefix, util::FRAG_SUFFIX)?;

        let ret = Self {
            prefix,
            camelcase,
            uppercase,
            dir,
            includes,
            variants,
        };

        Ok(ret)
    }

    pub fn get_path(&self, suffix: &str) -> PathBuf {
        self.dir.join(format!("{}{}", self.prefix, suffix))
    }
}

pub fn get_shader_info(path: PathBuf) -> Result<Vec<ShaderInfo>, Box<dyn Error>> {
    let mut shader_prefixes = vec![];

    let shaders_dir = std::fs::read_dir(&path)?;

    for shader_name in shaders_dir
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| Some(e.file_name().to_string_lossy().to_string()))
        .filter(|e| e.ends_with(util::VERT_SUFFIX))
    {
        let shader_prefix_len = shader_name.len() - util::VERT_SUFFIX.len();
        let shader_prefix = &shader_name[0..shader_prefix_len];
        shader_prefixes.push(shader_prefix.to_string());
    }

    let ret = shader_prefixes
        .into_iter()
        .map(|prefix| ShaderInfo::new(prefix, path.clone()).unwrap())
        .collect();

    Ok(ret)
}
