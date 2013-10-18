#version 150
out vec4 out_color;
in vec4 vert_color;
in vec2 vert_texcoords;

uniform sampler2DRect tex;

void main() {
    out_color = vec4(0.0,0.0,0.0,1.0-texture(tex, vert_texcoords).r)+vert_color;
}
