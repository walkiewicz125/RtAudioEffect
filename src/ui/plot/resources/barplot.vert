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

layout(std140, binding = 3) uniform MinMax
{
  vec2 min_max;
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

const vec4 bar_colors[6] = vec4[6](
    vec4(1.0, 0.0, 0.0, 1.0),
    vec4(0.0, 1.0, 0.0, 1.0),
    vec4(0.0, 0.0, 1.0, 1.0),

    vec4(1.0, 0.0, 0.4, 1.0),
    vec4(0.4, 1.0, 0.0, 1.0),
    vec4(0.0, 0.4, 1.0, 1.0)
);
float log10(float x)
{
    return log(x);//log(x) * inv_ln_of_10;
}
float log_of_bar(int bar_id)
{
    return log10(max(1.0, float(bar_id)))/log10(bar_count);
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
// make function to convert linear value to mix of colors 1-5
vec4 color_map1(float value)
{
    vec4 color1 = vec4(0.0, 0.0, 1.0, 0.0);
    vec4 color2 = vec4(0.0, 1.0, 1.0, 1.0);
    vec4 color3 = vec4(0.0, 1.0, 0.0, 1.0);
    vec4 color4 = vec4(1.0, 1.0, 0.0, 1.0);
    vec4 color5 = vec4(1.0, 0.0, 0.0, 1.0);

    if (value < 0.25)
        return mix(color1, color2, value * 4.0);
    else if (value < 0.5)
        return mix(color2, color3, (value - 0.25) * 4.0);
    else if (value < 0.75)
        return mix(color3, color4, (value - 0.5) * 4.0);
    else // if (value < 1.0)
        return mix(color4, color5, (value - 0.75) * 4.0);
}
void main()
{
    int bar_index = gl_VertexID / 6;
    int vertex_index = gl_VertexID % 6;

    vec2 vertex_pos = calculate_vertex_for_bar(bar_index, vertex_index);

    float mag = bar_values[bar_index];
    mag = clamp((mag - min_max.x) / (min_max.y - min_max.x), 0.0, 1.0);
    frag_color = color_map1(mag);
    gl_Position = projection * vec4(vertex_pos, 0.0, 1.0);
}
