﻿﻿﻿#version 330 core

uniform sampler2D Texture;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
} IN;

out vec4 Color;

void main()
{
    vec3 color = texture(Texture, IN.Uv).rgb;

    Color = IN.Color * vec4(color, 1.0);
}
