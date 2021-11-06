layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;
layout (location = 2) in vec2 in_tex_coords;
layout (location = 3) in vec3 in_normal;

uniform int instance_count;
uniform mat4 model;
uniform mat4 view;
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

    // Randomize a bit the position of each blade
    float instance_id = float(gl_InstanceID);
    float random_weight = 2.0;
    vec2 random_offset = random_weight * (rand2(vec2(instance_id)) - vec2(0.5));

    int stride = int(sqrt(instance_count));
    float column = float(gl_InstanceID % stride);
    float row = float(gl_InstanceID / stride);
    float scale = 0.25;
    float offset = 32.0;
    vec3 translation = vec3(0.0);
    translation.x = (column - offset) * scale + random_offset.x;
    translation.z = (row - offset) * scale + random_offset.y;

    float wind_speed = 1.0 / 32.0;
    float wind = worley(translation.xz / 32.0 + vec2(time) * wind_speed);
    color.rgb += wind * top_color_weight;

    // Rotation will affect only vertex at the top of the blade
    vec3 wind_direction = normalize(vec3(1.0, 0.0, -1.0));
    float wind_strength = 1.0;
    mat3 rotation = rot_from_axis_angle(wind_direction, wind * wind_strength * in_pos.y);

    // First rotate, then translate
    mat4 model_tmp = mat4(rotation) * model;
    model_tmp[3] = vec4(translation, 1.0);

    // Update normal with our new model matrix
    normal = inverse(transpose(mat3(model_tmp))) * in_normal;

    vec4 model_pos = model_tmp * vec4(in_pos, 1.0);

    pos_light_space = light_space * model_pos;

    gl_Position = proj * view * model_pos;
}
