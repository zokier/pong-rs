#version 150
in vec2 vertex;
uniform vec2 scale;
uniform vec2 position;
void main() {
    vec2 out_vert = vertex * scale;
    out_vert += position;
    gl_Position = vec4(out_vert, 0.0, 1.0);
}

