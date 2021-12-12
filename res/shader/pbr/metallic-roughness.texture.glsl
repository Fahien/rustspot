// Should be included after occlusion

uniform sampler2D mr_sampler;


vec3 get_metallic_roughness_occlusion(vec2 uv) {
    // G roughness
    // B metallic
    vec2 metallic_roughness = texture(mr_sampler, uv).gb;
    float roughness = metallic_roughness.x;
    float metallic = metallic_roughness.y;
    float occlusion = get_occlusion(uv);

    return vec3(occlusion, roughness, metallic);
}