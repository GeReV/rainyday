#version 330 core

uniform sampler2D TextureR;
uniform sampler2D TextureG;
uniform sampler2D TextureB;
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

const vec2 Kernel0Weights_RealX_ImY = vec2(0.411259, -0.548794);
const vec2 Kernel1Weights_RealX_ImY = vec2(0.513282, 4.561110);

//(Pr+Pi)*(Qr+Qi) = (Pr*Qr+Pr*Qi+Pi*Qr-Pi*Qi)
vec2 multComplex(vec2 p, vec2 q)
{
    return vec2(p.x*q.x-p.y*q.y, p.x*q.y+p.y*q.x);
}

void main()
{
    vec2 stepVal = 1.0 / Resolution;

    vec4 valR = vec4(0, 0, 0, 0);
    vec4 valG = vec4(0, 0, 0, 0);
    vec4 valB = vec4(0, 0, 0, 0);

    for (int i=-KERNEL_RADIUS; i <=KERNEL_RADIUS; ++i)
    {
        vec2 coords = IN.Uv + stepVal * vec2(0.0, float(i)) * FilterRadius;
        vec4 imageTexelR = texture(TextureR, coords);
        vec4 imageTexelG = texture(TextureG, coords);
        vec4 imageTexelB = texture(TextureB, coords);

        int pixel = int(i + KERNEL_RADIUS);

        vec4 c0_c1 = vec4(Kernel0[pixel].xy, Kernel1[pixel].xy);

        valR.xy += multComplex(imageTexelR.xy, c0_c1.xy);
        valR.zw += multComplex(imageTexelR.zw, c0_c1.zw);

        valG.xy += multComplex(imageTexelG.xy, c0_c1.xy);
        valG.zw += multComplex(imageTexelG.zw, c0_c1.zw);

        valB.xy += multComplex(imageTexelB.xy, c0_c1.xy);
        valB.zw += multComplex(imageTexelB.zw, c0_c1.zw);
    }

    float redChannel   = dot(valR.xy, Kernel0Weights_RealX_ImY)+dot(valR.zw, Kernel1Weights_RealX_ImY);
    float greenChannel = dot(valG.xy, Kernel0Weights_RealX_ImY)+dot(valG.zw, Kernel1Weights_RealX_ImY);
    float blueChannel  = dot(valB.xy, Kernel0Weights_RealX_ImY)+dot(valB.zw, Kernel1Weights_RealX_ImY);

    Color = vec4(vec3(redChannel, greenChannel, blueChannel), 1.0);
}
