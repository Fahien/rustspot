precision mediump int;
precision mediump float;
precision mediump sampler2DMS;

out vec4 out_color;

in vec3 color;
in vec2 tex_coords;

uniform vec2 extent;
uniform int tex_samples;
uniform sampler2DMS tex_sampler;

vec4 texture_ms(sampler2DMS texture, vec2 coords)
{
    vec4 color = vec4(0.0);
    ivec2 icoords = ivec2(coords * extent);

    for (int i = 0; i < tex_samples; ++i) {
        color += texelFetch(texture, icoords, i);
    }
    color /= float(tex_samples);

    return color;
}

void main() {
    vec4 depth = texture_ms(tex_sampler, tex_coords);
    out_color = vec4(color, 1.0) * vec4(depth.x, depth.x, depth.x, 1.0);
}
