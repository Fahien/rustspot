#version 320 es

out mediump vec4 out_color;

in mediump vec3 color;
in mediump vec2 tex_coords;

uniform sampler2D tex_sampler;

void main() {
    out_color = texture(tex_sampler, tex_coords);
}
