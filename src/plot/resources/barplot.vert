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

layout(std140, binding = 2) uniform ShaderStyle
{
  uint shader_style;
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
    return log(max(1.0, float(bar_id)))/log(bar_count);
}

float bar_width(int bar_id)
{
    float w = client_size.x * (log_of_bar(bar_id+1) - log_of_bar(bar_id));
    return w;
}

vec2 calculate_vertex_for_bar(int bar_id, int vertex_id)
{
    float x_offset = log_of_bar(bar_id) * (client_size.x);
    float vertex_x = x_offset + bar_vertices[vertex_id].x * bar_width(bar_id);

    float db_value = (20 / log(10)) * log(bar_values[bar_id]); // result -> db = [-inf; 0];
    float vertex_y = bar_vertices[vertex_id].y * client_size.y * (1.0 + db_value/100.0); // result -> db = [-db/100.0; 1.0]; -> basically from 0 to -100db

    return vec2(vertex_x, vertex_y);
}

void main()
{
	int bar_index = gl_VertexID / 6;
	int vertex_index = gl_VertexID % 6;

	vec2 vertex_pos = calculate_vertex_for_bar(bar_index, vertex_index);
    if (shader_style == 0)
    {
        frag_color = bar_colors[vertex_index];
    }
    else
    {
        frag_color = bar_colors[(vertex_index+1) % 6];
    }
	gl_Position = projection * vec4(vertex_pos, 0.0, 1.0);
}
