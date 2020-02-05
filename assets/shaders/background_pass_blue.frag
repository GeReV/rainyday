﻿﻿﻿#version 330 core

uniform sampler2D Texture;
uniform vec2 Resolution;
uniform vec4[17] Kernel0;
uniform vec4[17] Kernel1;
uniform float FilterRadius;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
} IN;

out vec4 Color;

const int KERNEL_RADIUS = 8;

void main()
{
    vec2 stepVal = 1.0 / Resolution;

    vec4 val = vec4(0, 0, 0, 0);

    for (int i=-KERNEL_RADIUS; i <=KERNEL_RADIUS; ++i)
    {
        vec2 coords = IN.Uv + stepVal * vec2(float(i), 0.0) * FilterRadius;

        float imageTexelB = texture(Texture, coords).b;

        int pixel = int(i + KERNEL_RADIUS);

        vec4 c0_c1 = vec4(Kernel0[pixel].xy, Kernel1[pixel].xy);

        val.xy += imageTexelB * c0_c1.xy;
        val.zw += imageTexelB * c0_c1.zw;
    }

    Color = val;
}
