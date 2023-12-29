#version 430 core

out vec3 frag_color;


layout(std140, binding = 0) uniform Projection
{
  mat4 projection;
};

layout(std140, binding = 1) uniform ClientSize
{
  vec2 client_size;
};

layout(std430, binding = 0) buffer bar_values_buffer
{
    uint bar_count;
	float[] bar_values;
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

const vec3 bar_colors[6] = vec3[6](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0),

    vec3(1.0, 0.0, 0.4),
    vec3(0.4, 1.0, 0.0),
    vec3(0.0, 0.4, 1.0)
);

float log_of_bar(int bar_id)
{
    return log(max(1.0, float(bar_id)))/log(2048);
}

float bar_width(int bar_id)
{
    float w = client_size.x * (log_of_bar(bar_id+1) - log_of_bar(bar_id));
    return w;
}

vec2 calculate_vertex_for_bar(int bar_id, int vertex_id)
{
    float bar_offset_x = log_of_bar(bar_id) * (client_size.x);//(client_size.x / bar_count);
    bar_offset_x = bar_offset_x * 24 / 20;
    vec2 vertex = vec2(bar_vertices[vertex_id].x * bar_width(bar_id) * 24 / 20, bar_vertices[vertex_id].y);
    float db = (20 / log(10)) * log(bar_values[bar_id]);
    // 0 max, -100db min
    db = (120+db)/100.0;
    vertex.y *=  (client_size.y) * db;
    // vertex.y *=  client_size.y;
    return vertex + vec2(bar_offset_x, 0);
}

void main()
{
	int bar_index = gl_VertexID / 6;
	int vertex_index = gl_VertexID % 6;

	vec2 vertex_pos = calculate_vertex_for_bar(bar_index, vertex_index);
    frag_color = bar_colors[vertex_index];

	gl_Position = projection * vec4(vertex_pos, 0.0, 1.0);
}
