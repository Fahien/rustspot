#version 320 es

precision mediump float;

out mediump vec4 out_color;

in mediump vec3 color;
in mediump vec2 tex_coords;

uniform sampler2D tex_sampler;
uniform int node_id;

void main() {
    out_color = texture(tex_sampler, tex_coords);
    if (node_id == 2) {
        out_color.rb = out_color.br;
    } else if (node_id == 3) {
        out_color.rg = out_color.gr;
    } else if (node_id == 4) {
        out_color.bg = out_color.gb;
    }
}
