precision mediump float;

out mediump vec4 out_color;

in highp vec3 position;
in mediump vec3 color;
in mediump vec2 tex_coords;

uniform vec3 horizon;
uniform vec3 zenit;

void main() {
    // Consider the angle between fragment position and horizontal plane
    vec3 pos = normalize(position);
    vec3 pos_proj = normalize(vec3(pos.x, 0.0, pos.z));
    float dot_value = dot(pos, pos_proj); // [0.0, 1.0]
    dot_value = pow(dot_value, 32.0); // Move values towards 0.0, or towards the zenit
    float sig = pos.y / abs(pos.y); // -1.0 if position y is negative, +1.0 otherwise
    float mix_factor = max(0.0, sig * (1.0 - dot_value));

    vec3 sky_color = mix(horizon, zenit, mix_factor);
    out_color = vec4(sky_color, 1.0);
}
