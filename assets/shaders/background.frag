﻿﻿﻿#version 330 core

uniform sampler2D Texture;
uniform vec2 Resolution;

in VS_OUTPUT {
    vec3 Position;
    vec4 Color;
    vec2 Uv;
} IN;

out vec4 Color;

float normpdf(in float x, in float sigma)
{
    return 0.39894 * exp(-0.5 * x * x / (sigma * sigma)) / sigma;
}

void main()
{
    //declare stuff
    const int mSize = 13;
    const int kSize = (mSize-1) / 2;

    float kernel[mSize];
    vec3 final_color = vec3(0.0);

    //create the 1-D kernel
    float sigma = 7.0;
    float Z = 0.0;

    for (int j = 0; j <= kSize; ++j)
    {
        kernel[kSize+j] = kernel[kSize-j] = normpdf(float(j), sigma);
    }

    //get the normalization factor (as the gaussian has been clamped)
    for (int j = 0; j < mSize; ++j)
    {
        Z += kernel[j];
    }

    //read out the texels
    for (int i=-kSize; i <= kSize; ++i)
    {
        for (int j=-kSize; j <= kSize; ++j)
        {
            final_color += kernel[kSize+j] * kernel[kSize+i] * texture(Texture, IN.Uv + vec2(float(i), float(j)) / Resolution).rgb;
        }
    }

    Color = vec4(final_color / (Z*Z), 1.0);
}
