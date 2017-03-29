#version 150 core
in vec2 pos;
in vec2 uv;

uniform sampler2D s_texture;
uniform vec4 color;

uniform vec2 vel;

out vec2 v_UV;
out vec2 v_pos;

void main() {
    v_UV = uv;
    gl_Position = vec4(pos, 0.0, 1.0);
    v_pos = pos;
}
