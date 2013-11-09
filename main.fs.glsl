#version 150
out vec4 out_color;
in vec4 vert_color;
in vec2 vert_texcoords;

uniform float texenabled;
uniform sampler2DRect tex;

void main() {
    // step() is a kludge as GL_NEAREST doesn't seem to work
    float alpha = step(0.5, texenabled * (1.0-texture(tex, vert_texcoords).r));
    out_color = vec4(0.0, 0.0, 0.0, alpha)+vert_color;
}
