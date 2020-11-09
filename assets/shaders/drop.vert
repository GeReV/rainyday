#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color;
layout (location = 2) in vec2 Uv;
layout (location = 3) in vec3 Offset;

uniform mat4 MVP;

out VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
    vec3 Offset;
} OUT;

void main()
{
    vec4 pos = vec4(Position.xy * Offset.z + Offset.xy, Position.z, 1.0);

    gl_Position = MVP * pos;

    OUT.Position = pos.xyz;
    OUT.Color = Color;
    OUT.Uv = Uv;
    OUT.Offset = Offset;
}
