#version 120
#include "logluv.frag"
precision lowp float;

varying float iTime;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    // Time varying pixel color
    //    vec3 col = 0.5 + 0.5 * cos(iTime + uv.xyx + vec3(0, 2, 4));

    // Output to screen
    vec4 color = vec4(uv, 0.0, 1.0);
    vec4 encoded = LogLuvEncode(color.rgb);
    gl_FragColor = encoded;
    //    vec3 decoded = LogLuvDecode(encoded);
    //    gl_FragColor = vec4(decoded, 1.0);
}