pub const COLOR_VERT: &'static str = "
#version 150 core
in vec4 color;
in vec2 pos;

out vec2 out_pos;
\
                                out vec4 v_Color;

void main() {
    v_Color = color;
    out_pos \
                                = pos;
    gl_Position = vec4(pos, 0.0, 1.0);
}";

pub const COLOR_FRAG: &'static str =
    "
#version 150 core
in vec4 v_Color;
in vec2 out_pos;

uniform float time;
uniform vec2 vel;

out vec4 o_Color;

float rand(vec2 co){
    return fract(sin(dot(co.xy \
     ,vec2(12.9898,78.233))) * 43758.5453);
}

float lig(vec4 c){
    return (c.r + c.g + c.b) / \
     3.0;
}

#define GRAIN_SCALE 50
#define GRAIN_MOVE 400
#define SPEED_MIN 1

#define \
     NOISE_OTHER (1.0 / 100.0)

void main() {

    float dir = sign(vel.x) * atan(-vel.y / 2.0, \
     abs(vel.x));
    float speed = sqrt(pow(vel.y, 2) + pow(vel.x, 2));
    if (speed < \
     SPEED_MIN) speed = SPEED_MIN;
    float xmod = floor((1 + out_pos.x) / pow(speed, 2) * \
     GRAIN_MOVE);
    vec2 roundPos =
        vec2(floor((out_pos.y - out_pos.x * dir) * \
     GRAIN_SCALE * sqrt(speed)),
             xmod + time);

    o_Color = vec4(0.0, 0.0, 0.0, \
     1.0);

    if (v_Color.r > rand(roundPos)) {
        o_Color.r = 1.0;
    }
    else {
        \
     o_Color.r = v_Color.r * NOISE_OTHER;
    }

    if (v_Color.g > rand(roundPos)) {
        \
     o_Color.g = 1.0;
    }
    else {
        o_Color.g = v_Color.g * NOISE_OTHER;
    }

    if \
     (v_Color.b > rand(roundPos)) {
        o_Color.b = 1.0;
    }
    else {
        o_Color.b = \
     v_Color.b * NOISE_OTHER;
    }
}";

pub const TEX_VERT: &'static str = "
#version 150 core
in vec2 pos;
in vec2 uv;

uniform sampler2D s_texture;
uniform vec4 color;

uniform vec2 vel;

out vec2 v_UV;

void main() {
    v_UV = uv;
    gl_Position = vec4(pos, 0.0, 1.0);
}";

pub const TEX_FRAG: &'static str =
    "
#version 150 core
in vec2 v_UV;

uniform sampler2D s_texture;
uniform vec4 color;

uniform float time_tex;
uniform vec2 vel;

out vec4 o_Color;

float rand(vec2 co){
    return fract(sin(dot(co.xy \
     ,vec2(12.9898,78.233))) * 43758.5453);
}

const float NOISE_SCALE = 32.0;

void main() {

    //o_Color = texture(s_texture, v_UV) * color;

    vec4 c = texture(s_texture, v_UV) * color;
    if (c.a < 1.0) {
        o_Color = c;
    }
    else {
        o_Color = c;
        //o_Color = vec4(rand(vec2(time_tex, time_tex + 1.0)), 1 , 1, 1);
        if (o_Color.b > 0 && o_Color.b < 1) {
            o_Color.r = 1;
            o_Color.g = 1;
            o_Color.b = 1;
            vec2 seed = vec2(time_tex + floor(v_UV.x * NOISE_SCALE) / NOISE_SCALE,
                             time_tex + floor(v_UV.y * NOISE_SCALE) / NOISE_SCALE);
            if (rand(seed) > c.b) {
                o_Color.b = 1;
                o_Color.g = 0;
                o_Color.r = 0;
            }
        }
    }
    /*
    o_Color = vec4(0.0, 0.0, 0.0, 1.0);
    vec2 seed = vec2(floor(v_UV.x * NOISE_SCALE) / NOISE_SCALE,
                     floor(v_UV.y * NOISE_SCALE) / NOISE_SCALE);
    vec2 r1 = vec2(seed.x + time, seed.y);
    vec2 r2 = vec2(seed.x - time, seed.y);
    vec2 r3 = vec2(seed.x, seed.y + time);

    if (c.r > rand(r1) * 0.95) {
        o_Color.r = 1.0;
    }
    if (c.g > rand(r2) * 0.95) {
        o_Color.g = 1.0;
    }
    if (c.b > rand(r3) * 0.95) {
        o_Color.b = 1.0;
    }
    */
}";
