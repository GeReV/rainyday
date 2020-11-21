#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 VertexColor;
layout (location = 2) in vec2 Uv;

uniform mat4 MVP;
uniform vec4 Color;

out VS_OUTPUT {
    vec4 Color;
} OUT;

void main()
{
    gl_Position = MVP * vec4(Position, 1.0);

    OUT.Color = VertexColor * Color;
}
