uniform sampler2D occlusion_sampler;

float get_occlusion(vec2 uv) {
    return texture(occlusion_sampler, uv).r;
}
