#version 320 es

out mediump vec4 out_color;

in mediump vec3 color;

void main() {
    out_color = vec4(color, 1.0);
}
