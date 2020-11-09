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
    // TODO: What's the best number for this?
    const float power = 1.0;

    // TODO: Find a formula for this?
    const float scale = 40.0;
    const float correction = 0.8;

    vec2 screen_coord_01 = gl_FragCoord.xy / Resolution;
    vec2 center_coord_01 = IN.Offset.xy / Resolution;

    vec2 uv = (IN.Uv * 2.0 - 1.0);
    float lensing = pow(length(uv), power);

    vec2 target_uv = (screen_coord_01 - center_coord_01) * scale * lensing + center_coord_01;

    // Offset to accommodate for texture edges.
    target_uv = target_uv * correction + (1.0 - correction) * 0.5;

    vec3 color = texture(Texture, target_uv).rgb;

    float opacity = smoothstep(0.0, 0.1, 1.0 - length(uv));

    Color = vec4(color, opacity);
}
