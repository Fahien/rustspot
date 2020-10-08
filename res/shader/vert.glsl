#version 320 es

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;

out vec3 color;

void main() {
    color = in_color;
    gl_Position = vec4(in_pos, 1.0);
}
