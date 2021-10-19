#version 450

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 color;

layout (set = 0, binding = 0) readonly buffer Ssbo {
    float data[];
} ssbo;

layout (set = 0, binding = 1) uniform Ubo {
    vec4 position;
    vec4 look_dir;
} ubo;

void main() {
    color = normalize(abs(ubo.position));
}
