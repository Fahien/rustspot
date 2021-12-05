layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;
layout (location = 2) in vec2 in_tex_coords;
layout (location = 3) in vec3 in_normal;

uniform int instance_count;
uniform mat4 model;
// There is a limit of 256 uniforms per shader
uniform mat4 models[128];
uniform mat4 view;
uniform mat3 billboard;
uniform mat4 proj;
uniform mat3 model_intr;
uniform mat4 light_space;
uniform float time;

out vec3 color;
out vec2 tex_coords;
out vec3 normal;
out vec4 pos_light_space;

// Simple noise
vec2 rand2(highp vec2 p) {
    return fract(
        sin(
            vec2(
                dot(p, vec2(228.23, 654.56)),
                dot(p, vec2(310.85, 142.98))
            )
        ) * 2431.843
    );
}

// Used to simulate the wind, where p is the position of the
// individual blade of grass translated by a time vector
float worley(vec2 p) {
    p.y += time / 16.0;
    p = 8.0 * p;
    vec2 int_part = floor(p);
    vec2 fract_part = fract(p);

    float dist = 1.0;

    // 3x3 grid
    for (int y = -1; y <= 1; ++y) {
        for (int x = -1; x <= 1; ++x) {
            vec2 cell = vec2(float(x), float(y));
            vec2 diff = cell + rand2(int_part + cell) - fract_part;
            dist = min(dist, smoothstep(0.0, 1.0, length(diff)));
        }
    }

    return dist;
}

// Used to bend the blade of grass
mat3 rot_from_axis_angle(vec3 axis, float angle) {
    float s = sin(angle);
    float c = cos(angle);
    float t = 1.0 - c;
    float x = axis.x;
    float y = axis.y;
    float z = axis.z;

    return mat3(
        vec3(t*x*x + c, t*x*y - s*z, t*x*z + s*y),
        vec3(t*x*y + s*z, t*y*y + c, t*y*z - s*x),
        vec3(t*x*z - s*y, t*y*z + s*z, t*z*z + c)
    );
}

void main() {
    color = in_color;

    // At the top is brighter
    float top_color_weight = 1.0 * in_pos.y;
    color.rgb += vec3(top_color_weight);

    tex_coords = in_tex_coords;

    mat4 instance_model = models[gl_InstanceID];
    vec3 translation = vec3(
        instance_model[3][0],
        instance_model[3][1],
        instance_model[3][2]
    );

    float wind_speed = 1.0 / 32.0;
    float wind = worley(translation.xz / 32.0 + vec2(time) * wind_speed);
    color.rgb += wind * top_color_weight;

    // Rotation will affect only vertex at the top of the blade
    vec3 wind_direction = normalize(vec3(1.0, 0.0, -1.0));
    float wind_strength = 1.0;
    mat3 rotation = rot_from_axis_angle(wind_direction, wind * wind_strength * in_pos.y);

    // Rotate to face the camera, like a billboard
    // Rotate first, then translate
    mat4 model_tmp = instance_model * mat4(billboard * rotation) * model;

    // Update normal with our new model matrix
    normal = inverse(transpose(mat3(model_tmp))) * normalize(in_normal);

    vec4 model_pos = model_tmp * vec4(in_pos, 1.0);

    pos_light_space = light_space * model_pos;

    gl_Position = proj * view * model_pos;
}
