#version 330 core

uniform sampler2D TexFace;

in VS_OUTPUT {
    vec4 Color;
    vec2 Uv;
    vec3 Normal;
    vec3 Position;
} IN;

out vec4 Color;

void main()
{
    vec3 color = texture(TexFace, IN.Uv).rgb;

    // normal
    vec3 normal = IN.Normal;

    // diffuse
    vec3 diffuse = color;

    Color = vec4(diffuse, 1.0);
}
