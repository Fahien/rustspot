// Custom build script

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

mod gen;

fn main() -> Result<(), Box<dyn Error>> {
    let mut code = String::from(gen::util::HEADER);

    // Read shaders
    let shaders_path = PathBuf::from("res/shader");
    let mut shader_infos = gen::shader_info::get_shader_info(shaders_path.clone())?;

    // Read shaders in subdirectories
    let shaders_dir = std::fs::read_dir(shaders_path)?;
    for dir in shaders_dir.filter_map(|x| {
        if let Ok(entry) = x {
            if let Ok(file_type) = entry.file_type() {
                // Ignore gen directories
                if file_type.is_dir() && entry.file_name() != "gen" {
                    return Some(entry)
                }
            }
        }
        None
    }) {
        shader_infos.append(&mut gen::shader_info::get_shader_info(dir.path())?);
    }

    // Generate enum with a value for each shader info
    code.push_str(&gen::shader_enum::generate(&shader_infos)?);

    // Generate create function which returns a vector with all shaders
    code.push_str(&gen::shader_create::generate(&shader_infos)?);
    
    // Generate structures for location and the actual shader for each info
    for info in &shader_infos {
        code.push_str(&gen::shader::generate(info)?);
    }

    // TODO: write in the target folder?
    let dest_path = Path::new("src/rustspot/shaders.rs");
    fs::write(dest_path, code)?;

    // Rerun build script if any shader changes or any of these sources change
    println!("cargo:rerun-if-changed=res/shader");
    println!("cargo:rerun-if-changed=src/build");

    Ok(())
}
