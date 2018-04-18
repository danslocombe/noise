#version 150 core
in vec2 v_UV;
in vec2 v_pos;

uniform sampler2D s_texture;
uniform vec4 color;

uniform mat4 replacement_colors;

uniform float time_tex;
//uniform vec2 pos_tex;

out vec4 o_Color;

vec3 color_palette(vec3 color) {
  vec4 homogeneous_color = vec4(color, 1.0);
  return (replacement_colors * homogeneous_color).rgb;
}

float rand(vec2 co){
  return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float morph(vec2 co) {
  return pow(abs(cos(co.x / 1200.0)), 0.5) + rand(co) / 200;
}

const float NOISE_SCALE = 32.0;

void main() {

  vec4 c_sample = texture(s_texture, v_UV) * color;
  vec4 color = c_sample;

  if (color.a >= 1.0) {
    if (color.b > 0 && color.b < 1) {
      color.r = 1;
      color.g = 1;
      color.b = 1;

      float seed_x = 5 * v_pos.x + time_tex + floor(v_UV.x * NOISE_SCALE) / NOISE_SCALE;
      float seed_y = time_tex + floor(v_UV.y * NOISE_SCALE) / NOISE_SCALE;
      vec2 seed = vec2(seed_x, seed_y);

      if (morph(seed) * 0.35 + 0.65 > c_sample.b) {
        color.b = 1;
        color.g = 0.33;
        color.r = 0.33;
      }
    }
  }
  o_Color = vec4(color_palette(color.rgb), color.a);
}
