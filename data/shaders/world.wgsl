struct VertexOutput {
    @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@location(0) position: vec3f) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4f(position, 1.0);

    return output;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 1.0, 1.0, 1.0);
}
