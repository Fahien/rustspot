// Default normal from vertex shader
vec3 get_normal(vec3 tangent, vec3 binormal, vec3 normal, vec2 uv) {
    return normalize(normal);
}
