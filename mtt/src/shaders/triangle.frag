#version 450

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 color;

layout (set = 0, binding = 0) readonly buffer Ssbo {
    float data[];
} ssbo;

void main() {
    color = vec4(vec3(ssbo.data[0]), 1.0);
}
