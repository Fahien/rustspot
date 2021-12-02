precision mediump float;

out mediump vec4 out_color;

in mediump vec3 color;
in mediump vec2 tex_coords;
in mediump vec3 normal;
in mediump vec4 pos_light_space;

uniform sampler2D tex_sampler;
uniform sampler2D shadow_sampler;

uniform vec3 light_color;
uniform vec3 light_direction;

float calculate_shadow(vec4 pos_light_space) {
    // Perspective divide so pos is in range [-1, 1]
    vec3 pos = pos_light_space.xyz / pos_light_space.w;
    // Now transform range to [0, 1] for shadow map
    pos = pos * 0.5 + 0.5;

    float closest_depth = texture(shadow_sampler, pos.xy).r;
    float current_depth = pos.z;

    vec3 normal = normalize(normal);
    vec3 light_dir = normalize(light_direction);
    float bias = max(0.005 * (1.0 - dot(normal, light_dir)), 0.0005);
    // Greater depth means it is further away
    float shadow = current_depth - bias > closest_depth ? 0.5 : 1.0;
    // 1.0 means no shadow
    return shadow;
}

void main() {
    float aw = 0.3;
    vec4 ambient = vec4(color * aw, 1.0);
    vec4 albedo = texture(tex_sampler, tex_coords);
    out_color = ambient * albedo;

    float dw = 1.0 - aw;
    float df = max(dot(normalize(normal), normalize(light_direction)), 0.0);
    vec4 diffuse = vec4(vec3(dw * df), 1.0);

    float shadow = calculate_shadow(pos_light_space);

    out_color += shadow * diffuse * vec4(light_color, 1.0) * albedo;
}
