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

layout(std430, binding = 0) buffer magnitude_data_buffer
{
    uint data_count;
    float[] magnitude_data;
};

// for each bar create two triangles:
/*
0        2   5
 +-------+  +
 |     /  / |
 |   /  /   |
 | /  /     |
 +  +-------+
1   3        4
*/

const vec2 bar_vertices[6] = vec2[6](
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),

    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

const vec4 rms_color = vec4(0.0, 0.0, 1.0, 1.0);
const vec4 peek_color = vec4(1.0, 0.0, 0.0, 1.0);

void main()
{
    int bar_index = gl_VertexID / 6;
    int vertex_index = gl_VertexID % 6;

    float bar_width = client_size.x / data_count;

    float x_offset = bar_index * bar_width;
    float vertex_x = x_offset + bar_vertices[vertex_index].x * bar_width;

    float vertex_y_amplitude = (2.0* (bar_vertices[vertex_index].y - 0.5)) * client_size.y * (0.0001 + magnitude_data[bar_index]);
    vertex_y_amplitude *= 1.0;
    float vertex_y = client_size.y / 2.0 + vertex_y_amplitude;

    frag_color = rms_color;
    gl_Position = projection * vec4(vertex_x, vertex_y, 0.0, 1.0);
}
