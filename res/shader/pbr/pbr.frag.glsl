// Metallic-Roughness

precision mediump float;

out mediump vec4 out_color;

// Fragment position in world space
in mediump vec3 world_pos;
in mediump vec3 color;
in mediump vec2 tex_coords;
in mediump vec3 normal;
in mediump vec4 pos_light_space;

uniform sampler2D tex_sampler;

#include "occlusion.glsl"
#include "metallic-roughness.glsl"
#include "shadow.glsl"

uniform vec3 light_color;
uniform vec3 light_direction;

// Camera position in world space
uniform vec3 cam_pos;

#define PI 3.14159265358979

#define MEDIUMP_FLT_MAX    65504.0
#define saturate_mediump(x) min(x, MEDIUMP_FLT_MAX)

// This models the distribution of the microfacet
// Surfaces are not smooth at the micro level, but made of a
// large number of randomly aligned planar surface fragments.
// This implementation is good for half-precision floats.
float distribution_ggx(float NoH, vec3 N, vec3 H, float roughness) {
    vec3 NxH = cross(N, H);
    float a = NoH * roughness;
    float k = roughness / (dot(NxH, NxH) + a * a);
    float d = k * k * (1.0 / PI);
    return saturate_mediump(d);
}

// This models the visibility of the microfacets, or occlusion or shadow-masking
float geometry_smith_ggx(float NoV, float NoL, float roughness) {
    float a = roughness;
    float GGXV = NoL * (NoV * (1.0 - a) + a);
    float GGXL = NoV * (NoL * (1.0 - a) + a);
    return 0.5 / (GGXV + GGXL);
}

vec3 fresnel_schlick(float cos_theta, vec3 f0) {
    float f = pow(1.0 - cos_theta, 5.0);
    return f + f0 * (1.0 - f);
}

void main() {
    vec4 albedo = texture(tex_sampler, tex_coords);
    // HDR?
    albedo.r = pow(albedo.r, 2.2);
    albedo.g = pow(albedo.g, 2.2);
    albedo.b = pow(albedo.b, 2.2);
    vec3 c = albedo.rgb;

    // TODO parameter?
    float reflectance = 0.5;
    vec3 metallic_roughness_occlusion = get_metallic_roughness_occlusion(tex_coords);
    float occlusion = metallic_roughness_occlusion.r;
    float roughness = metallic_roughness_occlusion.g;
    float metallic = metallic_roughness_occlusion.b;

    vec3 ambient = 0.125 * occlusion * c;

    vec3 N = normalize(normal);
    vec3 V = normalize(cam_pos - world_pos);
    float NoV = abs(dot(N, V)) + 1e-5;

    // Light out towards viewer
    vec3 Lo = vec3(0.0);

    // TODO for each light
    vec3 L = normalize(light_direction);
    vec3 H = normalize(V + L);
    float NoH = clamp(dot(N, H), 0.0, 1.0);
    float NoL = clamp(dot(N, L), 0.0, 1.0);
    float LoH = clamp(dot(L, H), 0.0, 1.0);

    // No attenuation for directional light
    vec3 radiance = 8.0 * light_color;

    // Frenel-Schlick
    vec3 f0 = vec3(0.16 * reflectance * reflectance * (1.0 - metallic)) + c * metallic;
    vec3 F = fresnel_schlick(LoH, f0);

    // Distribution of microfacets
    float D = distribution_ggx(NoH, N, H, roughness);

    // Visibility of microfacets
    float G = geometry_smith_ggx(NoV, NoL, roughness);

    // Cook-torrance specular microfacet model
    vec3 Fr = (D * G) * F;

    // Lambertian diffuse model
    // Pure metallic materials have no subsurface scattering
    vec3 Fd = ((1.0 - metallic) * c) / PI;

    Lo += (Fd + Fr) * radiance * NoL;
    // TODO end for each light

    vec3 color = ambient + Lo;

    // Shadow factor for unique directional light
    float shadow = calculate_shadow(pos_light_space, NoL);
    color = shadow * color;

    // HDR? Gamma correction?
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    out_color.rgb = color;
    out_color.a = albedo.a;
}
