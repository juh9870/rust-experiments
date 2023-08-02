#version 120
#include "logluv.frag"
precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    vec4 FragColor = texture2D(Texture, uv);
    //    float brightness = dot(FragColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    //    vec4 BrightColor;
    //    if (brightness > 0.5)
    //    BrightColor = vec4(FragColor.rgb, 1.0);
    //    else
    //    BrightColor = vec4(0.0, 0.0, 0.0, 1.0);
    //    gl_FragColor = BrightColor;
    gl_FragColor = FragColor;
    //    vec4 col = texture2D(Texture, uv);
    //    col = col - vec4(0);
    //    gl_FragColor = col;
}