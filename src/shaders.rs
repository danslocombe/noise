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
uniform vec2 vel;

out vec4 o_Color;

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float lig(vec4 c){
    return (c.r + c.g + c.b) / 3.0;
}

#define GRAIN_SCALE 50
#define GRAIN_MOVE 400
#define SPEED_MIN 1

void main() {

    float dir = sign(vel.x) * atan(-vel.y / 2.0, abs(vel.x));
    float speed = sqrt(pow(vel.y, 2) + pow(vel.x, 2));
    if (speed < SPEED_MIN) speed = SPEED_MIN;
    float xmod = floor((1 + out_pos.x) / pow(speed, 2) * GRAIN_MOVE);
    vec2 roundPos = 
        vec2(floor((out_pos.y - out_pos.x * dir) * GRAIN_SCALE * sqrt(speed)), 
             xmod + time);

    o_Color = vec4(0.0, 0.0, 0.0, 1.0);

    if (v_Color.r < rand(roundPos)) {
        o_Color.r = 1.0;
    }
    else {
        o_Color.r = o_Color.r / 2.0;
    }

    if (v_Color.g < rand(roundPos)) {
        o_Color.g = 1.0;
    }
    else {
        o_Color.b = o_Color.b / 2.0;
    }

    if (v_Color.b < rand(roundPos)) {
        o_Color.b = 1.0;
    }
    else {
        o_Color.g = o_Color.g / 2.0;
    }
}";
