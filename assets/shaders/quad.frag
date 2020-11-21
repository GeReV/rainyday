#version 330 core

uniform sampler2D Texture;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
} IN;

out vec4 Color;

void main()
{
    vec4 tex = texture(Texture, IN.Uv);

    Color = tex * IN.Color;
}
