#version 150

in vec2 vertex;
out vec4 vert_color;

uniform vec4 color;
uniform vec2 scale;
uniform vec2 position;
uniform vec2 window;

void main() {
    vec2 worldspace_size = vec2(4.0, 3.0);
    vec2 worldspace_origin = vec2(
            (0.9*window.y*worldspace_size.x/worldspace_size.y)/window.x,
            0.9) * -1.0;
    vec2 worldspace_scale = (worldspace_origin * -2.0)/worldspace_size;
    vec2 out_vert = vertex * scale;
    out_vert += position;
    out_vert *= worldspace_scale;
    out_vert += worldspace_origin;
    gl_Position = vec4(out_vert, 0.0, 1.0);
    vert_color = color;
}

