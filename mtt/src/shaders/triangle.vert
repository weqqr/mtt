#version 450

void main()
{
    vec2 position = vec2(
        ((gl_VertexIndex & 1) << 2) - 1,
        ((gl_VertexIndex & 2) << 1) - 1
    ) / 2;

    gl_Position = vec4(position, 0, 1);
}
