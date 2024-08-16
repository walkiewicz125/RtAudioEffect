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
  uint bar_count;
  float scale;
};

layout(std430, binding = 1) buffer magnitudes_buffer
{
    uint magnitude_count;
    float[] magnitudes;
};
layout (location = 0) in vec2 vertex;

// TODO: add shaper values
// CALC segment height etc
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

float get_db_for_bar(int bar_id)
{
    int bar_id_checked = clamp(bar_id, 0, int(bar_count)-1);
    return (20 / log(10)) * log(magnitudes[bar_id_checked]); // result -> db = [-inf; 0];
}

/*
   Tension: 1 is high, 0 normal, -1 is low
   Bias: 0 is even,
         positive is towards first segment,
         negative towards the other
*/
float HermiteInterpolate(
    int bar_id,
    float mu,
    float tension,
    float bias)
{
    float m0,m1,mu2,mu3;
    float a0,a1,a2,a3;

    float y0 = get_db_for_bar(bar_id-1);
    float y1 = get_db_for_bar(bar_id);
    float y2 = get_db_for_bar(bar_id+1);
    float y3 = get_db_for_bar(bar_id+2);

    mu2 = mu * mu;
    mu3 = mu2 * mu;
    m0  = (y1-y0)*(1+bias)*(1-tension)/2;
    m0 += (y2-y1)*(1-bias)*(1-tension)/2;
    m1  = (y2-y1)*(1+bias)*(1-tension)/2;
    m1 += (y3-y2)*(1-bias)*(1-tension)/2;
    a0 =  2*mu3 - 3*mu2 + 1;
    a1 =    mu3 - 2*mu2 + mu;
    a2 =    mu3 -   mu2;
    a3 = -2*mu3 + 3*mu2;

    return(a0*y1+a1*m0+a2*m1+a3*y2);
}

vec2 calculate_vertex(int bar_id, int vertex_id)
{
    float x_offset = log_of_bar(bar_id) * (client_size.x);
    float vertex_x = x_offset + vertex.x * bar_width(bar_id);

    float intepolated_y = (vertex.y) * client_size.y * (1.0 + HermiteInterpolate(bar_id+1, vertex.x, 0.0, 0.0)/100.0) *scale; // result -> db = [-db/100.0; 1.0]; -> basically from 0 to -100db
    return vec2(vertex_x, intepolated_y);
}

// make function to convert linear value to mix of colors 1-5
vec4 color_map1(float value)
{
    vec4 color1 = vec4(0.0, 0.0, 1.0, 0.0);
    vec4 color2 = vec4(0.0, 1.0, 1.0, 0.5);
    vec4 color3 = vec4(0.0, 1.0, 0.0, 0.8);
    vec4 color4 = vec4(1.0, 0.8, 0.0, 1.0);
    vec4 color5 = vec4(1.0, 0.8, 0.8, 1.0);

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
    int bar_index = gl_InstanceID;
    int vertex_index = gl_VertexID;

    vec2 vertex_pos = calculate_vertex(bar_index, vertex_index);

    float mag_this = magnitudes[bar_index+0];
    float mag_next = magnitudes[bar_index+1];
    float mag_next2 = magnitudes[bar_index+2];
    mag_this = clamp((mag_this - min_max.x) / (min_max.y - min_max.x), 0.0, 1.0);
    mag_next = clamp((mag_next - min_max.x) / (min_max.y - min_max.x), 0.0, 1.0);
    mag_next2 = clamp((mag_next2 - min_max.x) / (min_max.y - min_max.x), 0.0, 1.0);
    vec4 color_this = color_map1(mag_this);
    vec4 color_next = color_map1(mag_next);
    vec4 color_next2 = color_map1(mag_next2);

    frag_color = mix(color_next, color_next2, vertex.x);
    gl_Position = projection * vec4(vertex_pos, 0.0, 1.0);
}
