// Default include

uniform float metallic;
uniform float roughness;


vec3 get_metallic_roughness_occlusion(vec2 uv) {
    // Default to no occlusion
    float occlusion = get_occlusion(uv);
    return vec3(occlusion, roughness, metallic);
}
