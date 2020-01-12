#version 330 core

uniform sampler2D Texture;
uniform vec2 Resolution;
uniform int Time;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
} IN;

out vec4 Color;

void main()
{
    const float power = 0.66;
    const float scale = 0.3;

    float aspect_correction = Resolution.y / Resolution.x;

    vec2 offset = vec2(0.0, 0.0);

    vec2 uv = (IN.Uv * 2.0 - 1.0);

    vec2 lens_uv = uv * vec2(aspect_correction, 1.0);
    vec2 target_uv = (lens_uv * pow(length(lens_uv), power));

    vec3 color = texture(Texture, ((target_uv + 1.0) * 0.5) + offset).rgb;

    float opacity = smoothstep(0.0, 0.1, 1.0 - length(uv));

    Color = vec4(color, opacity);
}
