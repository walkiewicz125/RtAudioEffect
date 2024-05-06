#version 430 core

out vec4 frag_color;

layout(std140, binding = 0) uniform Projection
{
    mat4 projection;
};

layout(std140, binding = 1) uniform ClientSize
{
    vec2 client_size;
};

layout(std140, binding = 3) uniform ShaderParametters
{
    float line_width;
    vec4 line_color;
};

layout(std430, binding = 0) buffer points_bufer
{
    uint point_count;
    vec2[] points;
};
/*
LINE            CONNECTION
0        2  5   6        7   11
 +-------+  +   +-------+  +
 |     /  / |   |     /  / |
 |   /  /   |   |   /  /   |
 | /  /     |   | /  /     |
 +  +-------+   +  +-------+
1   3       4   8  9       10
*/
// resources: lines_vertex_shader.drawio

vec2 make_line(int point_index, int vertex_index)
{
    vec2 A = points[point_index];
    vec2 B = points[point_index + 1];

    vec2 AB = normalize(B - A);

    vec2 normalAB = vec2(-AB.y, AB.x);
    vec2 vertex_pos = vec2(0, 0);
    switch (vertex_index)
    {
    case 0:
        vertex_pos = A + normalAB * line_width/2.0;
        break;

    case 1:
        vertex_pos = A - normalAB * line_width/2.0;
        break;

    case 2:
        vertex_pos = B + normalAB * line_width/2.0;
        break;

    case 3:
        vertex_pos = A - normalAB * line_width/2.0;
        break;

    case 4:
        vertex_pos = B + normalAB * line_width/2.0;
        break;

    case 5:
        vertex_pos = B - normalAB * line_width/2.0;
        break;
    }

    return vertex_pos;
}

vec2 make_connection(int point_index, int vertex_index)
{
    vec2 A = points[point_index];
    vec2 B = points[point_index + 1];
    vec2 C = points[point_index + 2];

    vec2 AB = normalize(B - A);
    vec2 BC = normalize(C - B);

    vec2 normalAB = vec2(-AB.y, AB.x);
    vec2 normalBC = vec2(-BC.y, BC.x);
    vec2 miterB = normalize(normalAB + normalBC);
    if ((normalAB.x*normalBC.y - normalAB.y*normalBC.x) < 0 )
    {
        miterB *= -1.0;
        normalAB *= -1.0;
        normalBC *= -1.0;
    }

    vec2 vertex_pos = vec2(0, 0);
    switch (vertex_index)
    {
    case 0:
        vertex_pos = B - normalAB * line_width/2.0;
        break;

    case 1:
        vertex_pos = B;
        break;

    case 2:
        vertex_pos = B - miterB * line_width/2.0;
        break;

    case 3:
        vertex_pos = B - miterB * line_width/2.0;
        break;

    case 4:
        vertex_pos = B;
        break;

    case 5:
        vertex_pos = B - normalBC * line_width/2.0;
        break;
    }
    return vertex_pos;
}

void main()
{
    const int point_index = (gl_VertexID / 12);
    int vertex_index = gl_VertexID % 6;
    int segment_index = (gl_VertexID % 12/*0-11*/) / 6; // -> 0 = line, 1 = connection

    vec2 vertex_pos = vec2(0, 0);
    if (segment_index == 0)
    {
        vertex_pos = make_line(point_index, vertex_index);
    }
    else
    {
        vertex_pos = make_connection(point_index, vertex_index);
    }

    frag_color = line_color;

    vertex_pos.x /= client_size.x;
    vertex_pos.y /= client_size.y;
    gl_Position = vec4(vertex_pos, 0.0, 1.0);
}
