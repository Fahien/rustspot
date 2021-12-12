pub const VERT_SUFFIX: &str = ".vert.glsl";
pub const FRAG_SUFFIX: &str = ".frag.glsl";

pub const HEADER: &str = r#"// Generated code, do not modify.
use crate::*;

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use nalgebra as na;

"#;

/// Converts a string to camelcase, removing all `-` and `_` characters.
pub fn to_camelcase(name: &str) -> String {
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
