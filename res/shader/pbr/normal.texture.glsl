uniform sampler2D normal_sampler;

vec3 get_normal(vec3 tangent, vec3 binormal, vec3 normal, vec2 uv) {
    vec3 sampled_normal = texture(normal_sampler, uv).rgb;
    sampled_normal = sampled_normal * 2.0 - 1.0;
    mat3 TBN = mat3(normalize(tangent), normalize(binormal), normalize(normal));
    return normalize(TBN * sampled_normal);
}
