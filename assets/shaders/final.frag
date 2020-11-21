#version 330 core

uniform sampler2D Texture0;
uniform sampler2D Texture1;
uniform sampler2D Mask;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
} IN;

out vec4 Color;

void main()
{
    vec4 mask = texture(Mask, IN.Uv);
    vec4 tex0 = texture(Texture0, IN.Uv);
    vec4 tex1 = texture(Texture1, IN.Uv);

    Color = mix(tex0, tex1, clamp(mask.r, 0.0, 1.0)) * IN.Color;
}
