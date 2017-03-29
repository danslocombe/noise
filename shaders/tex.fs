#version 150 core
in vec2 v_UV;
in vec2 v_pos;

uniform sampler2D s_texture;
uniform vec4 color;

uniform float time_tex;
uniform vec2 pos_tex;

out vec4 o_Color;

float rand(vec2 co){
  return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float morph(vec2 co) {
  return pow(abs(cos(co.x / 1200.0)), 0.5) + rand(co) / 200;
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
    if (o_Color.b > 0 && o_Color.b < 1) {
      o_Color.r = 1;
      o_Color.g = 1;
      o_Color.b = 1;
      vec2 seed = vec2(pos_tex.x +  5 * v_pos.x + time_tex + 
                  floor(v_UV.x * NOISE_SCALE) / NOISE_SCALE, time_tex + floor(v_UV.y * NOISE_SCALE) / NOISE_SCALE);
      if (morph(seed) * 0.35 + 0.65 > c.b) {
        o_Color.b = 1;
        o_Color.g = 0.33;
        o_Color.r = 0.33;
      }
   }
  }
}
