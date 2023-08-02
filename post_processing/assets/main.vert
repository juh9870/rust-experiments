#version 120
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec4 time;
varying lowp vec2 uv;
varying lowp vec4 color;

varying lowp vec2 uv_screen;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
    time = _Time;

    vec3 ndc = gl_Position.xyz / gl_Position.w; //perspective divide/normalize
    uv_screen = ndc.xy * 0.5 + 0.5; //ndc is -1 to 1 in GL. scale for 0 to 1
}