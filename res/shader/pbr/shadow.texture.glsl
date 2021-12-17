uniform sampler2D shadow_sampler;

float calculate_shadow(vec4 pos_light_space, float NoL) {
    // Perspective divide so pos is in range [-1, 1]
    vec3 pos = pos_light_space.xyz / pos_light_space.w;
    // Now transform range to [0, 1] for shadow map
    pos = pos * 0.5 + 0.5;
    if (pos.z > 1.0) {
        return 1.0;
    }

    float closest_depth = texture(shadow_sampler, pos.xy).r;
    float current_depth = pos.z;

    float bias = max(0.05 * (1.0 - NoL), 0.005);
    // Greater depth means it is further away
    float shadow = current_depth - bias > closest_depth ? 1.0 : 0.5;

    vec2 texel_size = vec2(1.0 / 512.0);
    for(int x = -1; x <= 1; ++x) {
        for(int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadow_sampler, pos.xy + vec2(float(x), float(y)) * texel_size).r;
            shadow += current_depth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow /= 18.0;

    // 1.0 means no shadow
    return  1.0 - shadow;
}