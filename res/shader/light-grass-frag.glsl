precision mediump float;

out mediump vec4 out_color;

in mediump vec3 color;
in mediump vec2 tex_coords;
in mediump vec3 normal;
in mediump vec4 pos_light_space;

uniform sampler2D tex_sampler;

uniform vec3 light_color;
uniform vec3 light_direction;

void main() {
    float aw = 0.3;
    vec4 ambient = vec4(color * aw, 1.0);
    vec4 albedo = texture(tex_sampler, tex_coords);
    out_color = ambient * albedo;

    float dw = 1.0 - aw;
    float df = max(dot(normalize(normal), normalize(light_direction)), 0.0);
    vec4 diffuse = vec4(vec3(dw * df), 1.0);

    out_color += diffuse * vec4(light_color, 1.0) * albedo;
}
