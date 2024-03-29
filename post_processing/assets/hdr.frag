#version 120
#include "logluv.frag"
precision lowp float;

varying vec4 color;
varying vec2 uv;

varying lowp vec2 uv_screen;
uniform sampler2D _ScreenTexture;
uniform sampler2D Texture;

varying lowp vec4 time;

// https://www.shadertoy.com/view/XtlSD7

vec2 CRTCurveUV(vec2 uv)
{
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs(uv.yx) / vec2(6.0, 4.0);
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void DrawVignette(inout vec3 color, vec2 uv)
{
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    vignette = clamp(pow(16.0 * vignette, 0.3), 0.0, 1.0);
    color *= vignette;
}


void DrawScanline(inout vec3 color, vec2 uv)
{
    float iTime = 0.1;
    float scanline = clamp(0.95 + 0.05 * cos(3.14 * (uv.y + 0.008 * iTime) * 240.0 * 1.0), 0.0, 1.0);
    float grille = 0.85 + 0.15 * clamp(1.5 * cos(3.14 * uv.x * 640.0 * 1.0), 0.0, 1.0);
    color *= scanline * grille * 1.2;
}

void main() {
    vec4 bg = texture2D(_ScreenTexture, uv_screen);
    vec4 col = texture2D(Texture, uv) * color;

    float factor = clamp(time.y + 0.5, 0.0, 1.0);
    //    bg = (vec4(1) - bg) * factor + bg * (1.0 - factor);
    //    bg.a = 1.0;

    vec3 decoded = LogLuvDecode(bg);
    gl_FragColor = vec4(decoded, 1.0);
    //    gl_FragColor = vec4(LogLuvDecode(bg), 1.0);
    //    gl_FragColor = LogLuvEncode(bg.xyz);
    //    vec2 crtUV = CRTCurveUV(uv);
    //
    //    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    //
    //    if (crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0)
    //    {
    //        res = vec3(0.0, 0.0, 0.0);
    //    }
    //    DrawVignette(res, crtUV);
    //    DrawScanline(res, uv);
    //    gl_FragColor = vec4(res, 1.0);

}