#version 150 core
in vec4 v_Color;
in vec2 out_pos;

uniform float time;
uniform vec2 vel;
uniform mat4 replacement_colors;


out vec4 o_Color;

vec3 color_palette(vec3 color) {
  vec4 homogeneous_color = vec4(color, 1.0);
  return (replacement_colors * homogeneous_color).rgb;
}

float rand(vec2 co){
    return fract(sin(dot(co.xy,vec2(12.9898,78.233))) * 43758.5453);
}

float lig(vec4 c){
    return (c.r + c.g + c.b) / 3.0;
}

#define GRAIN_SCALE 50
#define GRAIN_MOVE 400
#define SPEED_MIN 1

#define NOISE_OTHER (1.0 / 100.0)

void main() {

    float dir = sign(vel.x) * atan(-vel.y / 2.0, abs(vel.x));
    float speed = sqrt(pow(vel.y, 2) + pow(vel.x, 2));

    if (speed < SPEED_MIN) {
      speed = SPEED_MIN;
    }
    float xmod = floor((1 + out_pos.x) / pow(speed, 2) * GRAIN_MOVE);
    vec2 roundPos = vec2(floor((out_pos.y - out_pos.x * dir) 
                  * GRAIN_SCALE * sqrt(speed)), xmod + time);

    o_Color = vec4(0.0, 0.0, 0.0, 1.0);

    if (v_Color.r > rand(roundPos)) {
        o_Color.r = 1.0;
    }
    else {
     o_Color.r = v_Color.r * NOISE_OTHER;
    }

    if (v_Color.g > rand(roundPos)) {
     o_Color.g = 1.0;
    }
    else {
        o_Color.g = v_Color.g * NOISE_OTHER;
    }

    if (v_Color.b > rand(roundPos)) {
        o_Color.b = 1.0;
    }
    else {
        o_Color.b = v_Color.b * NOISE_OTHER;
    }
    o_Color = vec4(color_palette(o_Color.rgb), o_Color.a);
}
