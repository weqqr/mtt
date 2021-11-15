#version 450

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 color;

layout (set = 0, binding = 0) uniform Uniforms {
    float time;
    float aspect_ratio;
} uniforms;

const float PI = 3.1415926;
const float RADIUS = 0.05;
const float INNER_RADIUS = 0.03;
const float ANGULAR_SIZE = PI;

void main() {
    float theta = uniforms.time * 4;
    color = vec4(0, 0, 0, 1);

    // Move origin to the center and correct for aspect ratio
    vec2 uv = (v_uv - 0.5) * 2 * vec2(uniforms.aspect_ratio, 1);

    // Rotate the whole frame
    mat2 rotation = mat2(cos(theta), sin(theta), -sin(theta), cos(theta));
    uv *= rotation;

    float angle = atan(uv.y, uv.x);

    float distance_from_center = dot(uv, uv);
    bool inside_outer_circle = distance_from_center < RADIUS * RADIUS;
    bool outside_inner_circle = distance_from_center > INNER_RADIUS * INNER_RADIUS;
    bool in_sector = angle < PI;

    if (inside_outer_circle && outside_inner_circle && in_sector) {
        color = vec4(smoothstep(vec3(0), vec3(1), vec3(angle / ANGULAR_SIZE)), 1);
    }
}
