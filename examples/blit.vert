#version 450

layout(location = 0) out vec2 v_tex_coord;

void main() {
    switch (gl_VertexIndex % 3) {
    case 0:
        v_tex_coord = vec2(0, 2);
        gl_Position = vec4(-1, 3, 0, 1);
        break;
    case 1:
        v_tex_coord = vec2(0, 0);
        gl_Position = vec4(-1, -1, 0, 1);
        break;
    case 2:
        gl_Position = vec4(3, -1, 0, 1);
        v_tex_coord = vec2(2, 0);
        break;
    }
}
