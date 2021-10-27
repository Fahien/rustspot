layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;
layout (location = 2) in vec2 in_tex_coords;
layout (location = 3) in vec3 in_normal;

uniform mat4 model;

out vec3 position;
out vec3 color;
out vec2 tex_coords;

void main() {
    color = in_color;
    tex_coords = in_tex_coords;

    // Send position translated along Z and rotated to face the camera
    position = mat3(model) * vec3(2.0 * in_pos.xy, -2.0);

    // Use w as z value to put the fragment at depth 1.0
    gl_Position = vec4(2.0 * in_pos, 1.0).xyww;
}
