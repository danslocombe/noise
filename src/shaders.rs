pub const VERT : &'static str = "
#version 150 core
in vec4 color;
in vec2 pos;

out vec2 out_pos;
out vec4 v_Color;

void main() {
    v_Color = color;
    out_pos = pos;
    gl_Position = vec4(pos, 0.0, 1.0);
}";

pub const FRAG : &'static str = "
#version 150 core
in vec4 v_Color;
in vec2 out_pos;

uniform float time;
out vec4 o_Color;

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float lig(vec4 c){
    return (c.r + c.g + c.b) / 3.0;
}

void main() {
    //o_Color = v_Color;
    if (lig(v_Color) < rand(vec2(out_pos.x - time, out_pos.y + time))) {
        o_Color = vec4(0.0, 0.0, 0.0, 1.0);
    }
    else {
        o_Color = vec4(1.0, 1.0, 1.0, 0.0);
    }
}";
