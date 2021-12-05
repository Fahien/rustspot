precision mediump float;

out mediump vec4 out_color;

// Fragment position in world space
in mediump vec3 world_pos;
in mediump vec3 color;
in mediump vec2 tex_coords;
in mediump vec3 normal;
in mediump vec4 pos_light_space;

uniform sampler2D tex_sampler;
uniform float metallic;
uniform float roughness;

uniform sampler2D shadow_sampler;

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
float distribution_ggx(float NoH, vec3 normal, vec3 half_vec, float roughness) {
    vec3 NxH = cross(normal, half_vec);
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

float calculate_shadow(vec4 pos_light_space) {
    // Perspective divide so pos is in range [-1, 1]
    vec3 pos = pos_light_space.xyz / pos_light_space.w;
    // Now transform range to [0, 1] for shadow map
    pos = pos * 0.5 + 0.5;

    float closest_depth = texture(shadow_sampler, pos.xy).r;
    float current_depth = pos.z;

    vec3 normal = normalize(normal);
    vec3 light_dir = normalize(light_direction);
    float bias = max(0.005 * (1.0 - dot(normal, light_dir)), 0.0005);
    // Greater depth means it is further away
    float shadow = current_depth - bias > closest_depth ? 0.5 : 1.0;
    // 1.0 means no shadow
    return shadow;
}

void main() {
    vec3 normal = normalize(normal);
    vec3 view_vec = normalize(cam_pos - world_pos);

    // Light out towards viewer
    vec3 Lo = vec3(0.0);

    // TODO for each light
    vec3 light_vec = normalize(light_direction);
    vec3 half_vec = normalize(view_vec + light_vec);
    float NoV = abs(dot(normal, view_vec)) + 1e-5;
    float NoH = clamp(dot(normal, half_vec), 0.0, 1.0);
    float NoL = clamp(dot(normal, light_vec), 0.0, 1.0);
    float HoL = clamp(dot(half_vec, light_vec), 0.0, 1.0);

    // No attenuation for directional light
    vec3 radiance = 4.0 * light_color;

    vec3 aw = vec3(0.03);
    vec4 ambient = vec4(aw, 1.0);
    vec4 albedo = texture(tex_sampler, tex_coords);
    // HDR?
    albedo.r = pow(albedo.r, 2.2);
    albedo.g = pow(albedo.g, 2.2);
    albedo.b = pow(albedo.b, 2.2);
    vec3 c = albedo.rgb;

    // Frenel-shlick
    // TODO parameter?
    float reflectance = 0.5;
    vec3 f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + c * metallic;
    vec3 F  = fresnel_schlick(HoL, f0);

    // Distribution of microfacets
    float D = distribution_ggx(NoH, normal, half_vec, roughness);

    // Visibility of microfacets
    float G = geometry_smith_ggx(NoV, NoL, roughness);

    // Cook-torrance specular microfacet model
    vec3 numerator    = (D * G) * F;
    float denominator = 4.0 * NoV * NoL + 0.0001;
    vec3 specular     = numerator / denominator;

    // Lambertian diffuse model
    // Pure metallic materials have no subsurface scattering
    vec3 diffuse = ((1.0 - metallic) * c) / PI;

    // Lighting!
    Lo += (diffuse + specular) * radiance * NoL;

    // Shadow factor
    float shadow = calculate_shadow(pos_light_space);

    out_color = ambient * albedo + shadow * vec4(Lo, 1.0);

    // HDR? Gamma correction?
    vec3 color = out_color.rgb / (out_color.rgb + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    out_color.rgb = color;
}
