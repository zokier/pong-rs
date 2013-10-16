#version 150
in vec2 vertex;
out vec4 vert_color;
uniform vec4 color;
uniform vec2 scale;
uniform vec2 position;
void main() {
    // 540 = 90% of window height (600)
    // 720 = 4*540/3
    // 1024 = window width
    // 2 = screenspace height/width
    // 4 = worldspace width
    // 3 = worldspace height 
    vec2 worldspace_origin = vec2(-720.0/1024.0, -540/600.0);
    vec2 worldspace_scale = vec2((2.0*720.0)/(4.0*1024.0), (2.0*540.0)/(3.0*600.0));
    vec2 out_vert = vertex * scale;
    out_vert += position;
    out_vert *= worldspace_scale;
    out_vert += worldspace_origin;
    gl_Position = vec4(out_vert, 0.0, 1.0);
    vert_color = color;
}
