precision mediump float;

out mediump vec4 out_color;

in mediump vec3 color;
in mediump vec2 tex_coords;

uniform sampler2D tex_sampler;

struct DirectionalLight {
    vec3 color;
    vec3 direction;
};

uniform DirectionalLight directional_light;

void main() {
    out_color = vec4(directional_light.color, 1.0) * texture(tex_sampler, tex_coords);
}
