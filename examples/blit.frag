#version 450

layout(set = 0, binding = 0) uniform texture2D u_blit_texture;
layout(set = 0, binding = 1) uniform sampler u_blit_sampler;

layout(location = 0) in vec2 v_tex_coord;

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(texture(sampler2D(u_blit_texture, u_blit_sampler), v_tex_coord).ggg, 1);
}
