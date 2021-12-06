precision mediump int;
precision mediump float;
precision mediump sampler2D;

out vec4 out_color;

in vec3 color;
in vec2 tex_coords;

uniform sampler2D tex_sampler;


void main() {
    vec4 tex_color = texture(tex_sampler, tex_coords);
    out_color = vec4(color, 1.0) * tex_color;
}
