#version 430 core

out vec4 frag_color;

layout(location = 0)  in vec2 vertex;

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

layout(std430, binding = 2) buffer SpectrogramData
{
  uint width;
  uint length;
  float magnitude[];
};
const float inv_ln_of_10 = 1.0 / log(10.0);

float log10(float x)
{
    return log(x);//log(x) * inv_ln_of_10;
}

float log_of_bar(uint bar_id)
{
    return log10(max(1.0, float(bar_id)))/log10(width);
}

float bar_width(uint bar_id)
{
    float w = client_size.x * (log_of_bar(bar_id+1) - log_of_bar(bar_id));
    return w;
}


// make function to convert linear value to rgb rainbow
vec3 rainbow(float value)
{
    float h = (value) * 5.0;
    float r = clamp(3.0 - abs(h - 4.0), 0.0, 1.0);
    float g = clamp(2.0 - abs(h - 2.0), 0.0, 1.0);
    float b = clamp(2.0 - abs(h - 1.0), 0.0, 1.0);
    return vec3(r, g, b);
}

vec4 return_with_transparent_gradient(vec3 rgb)
{
    float alpha = (rgb.r + rgb.g + rgb.b) / 3.0;
    return vec4(rgb, alpha);
}

void main()
{
    uint freq_bar = gl_InstanceID % width;
    uint time_step = gl_InstanceID / width;

    float x_offset = log_of_bar(freq_bar) * (client_size.x);
    float vertex_x = x_offset + vertex.x * bar_width(freq_bar);

    vec2 out_pos = vec2(vertex_x, vertex.y + time_step);
    out_pos.y *= client_size.y / length;

    float mag = magnitude[gl_InstanceID];
    mag = clamp((mag - min_max.x) / (min_max.y - min_max.x), 0.0, 1.0);

    gl_Position = projection * vec4(out_pos, 0.0, 1.0);
    frag_color = vec4(return_with_transparent_gradient(rainbow(mag)));
}
