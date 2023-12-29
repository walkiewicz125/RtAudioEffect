#version 430 core

in vec3 frag_color;
out vec4 color_out;

void main()
{
    color_out = vec4(frag_color, 1.0);
}
