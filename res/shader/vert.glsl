#version 320 es

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;
layout (location = 2) in vec2 in_tex_coords;

layout (location = 0) uniform mat4 model;
layout (location = 1) uniform mat4 view;
layout (location = 2) uniform mat4 proj;

out vec3 color;
out vec2 tex_coords;

void main() {
    color = in_color;
    tex_coords = in_tex_coords;
    gl_Position = proj * view * model * vec4(in_pos, 1.0);
}
