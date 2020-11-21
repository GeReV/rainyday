#version 330 core

uniform sampler2D Texture;
uniform vec2 Resolution;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
    vec3 Offset;
} IN;

out vec4 Color;

void main()
{
    vec2 uv = (IN.Uv * 2.0 - 1.0);

    float opacity = smoothstep(0.0, 0.1, 1.0 - length(uv));

    Color = vec4(vec3(1.0), opacity);
}
