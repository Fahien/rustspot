layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;
layout (location = 2) in vec2 in_tex_coords;
layout (location = 3) in vec3 in_normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;
uniform mat3 model_intr;
uniform mat4 light_space;

out vec3 color;
out vec2 tex_coords;
out vec3 normal;
out vec4 pos_light_space;

void main() {
    color = in_color;
    tex_coords = in_tex_coords;
    normal = model_intr * in_normal;

    vec4 model_pos = model * vec4(in_pos, 1.0);

    pos_light_space = light_space * model_pos;

    gl_Position = proj * view * model_pos;
}
