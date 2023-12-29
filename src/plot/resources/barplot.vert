#version 330 core

layout (location = 0) in vec2 vertex_pos;
layout (location = 1) in vec3 bar_color;
layout (location = 2) in vec2 bar_position_and_size;

out vec3 fColor;

uniform mat4 projection;

void main()
{
    vec2 new_position = vertex_pos;
    new_position.x += bar_position_and_size.x;
    new_position.y *= bar_position_and_size.y;

    gl_Position = projection * vec4(new_position, 0.0, 1.0);;
    fColor = bar_color;
}
