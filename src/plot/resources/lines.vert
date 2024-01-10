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

layout(std140, binding = 3) uniform ShaderParametters
{
  float line_width;
};

layout(std430, binding = 0) buffer points_bufer
{
    uint point_count;
	  vec2[] points;
};
/*
0        2   5
 +-------+  +
 |     /  / |
 |   /  /   |
 | /  /     |
 +  +-------+
1   3        4
*/

void main() {
  const int point_index = 1 + (gl_VertexID / 6);

  vec2 A = points[point_index - 1];
  vec2 B = points[point_index];
  vec2 C = points[point_index + 1];
  vec2 D = points[point_index + 2];

  if (all(equal(A, B))) {
    A = B + normalize(B - C);
  }
  if (all(equal(C, D))) {
    D = C + normalize(C - B);
  }

  vec2 AB = normalize(B - A);
  vec2 BC = normalize(C - B);
  vec2 CD = normalize(D - C);

  vec2 tanAC = normalize(AB + BC);
  vec2 tanBD = normalize(BC + CD);

  vec2 miterB = vec2(-tanAC.y, tanAC.x);
  vec2 miterC = vec2(-tanBD.y, tanBD.x);

  vec2 normalA = vec2(-AB.y, AB.x);
  vec2 normalB = vec2(-BC.y, BC.x);


  miterB *= line_width / dot(miterB, normalA);
  miterC *= line_width / dot(miterC, normalB);


  vec2 vertex_pos = vec2(0, 0);
	int vertex_index = gl_VertexID % 6;


  switch (vertex_index) {
    case 0:
      vertex_pos = B - miterB;
      frag_color = vec3(1.0, 0.0, 0.0);
      break;

    case 1:
      vertex_pos = B + miterB;
      frag_color = vec3(0.0, 1.0, 0.0);
      break;

    case 2:
      vertex_pos = C - miterC;
      frag_color = vec3(0.0, 0.0, 1.0);
      break;

    case 3:
      vertex_pos = B + miterB;
      frag_color = vec3(1.0, 0.0, 0.4);
      break;

    case 4:
      vertex_pos = C + miterC;
      frag_color = vec3(0.4, 1.0, 0.0);
      break;

    case 5:
      vertex_pos = C - miterC;
      frag_color = vec3(0.0, 0.4, 1.0);
      break;
  }



  //frag_color = points[gl_VertexID].frag_color;
  // frag_color = vec4(1.0, 1.0, 1.0, 1.0);
  vertex_pos.x /= client_size.x;
  vertex_pos.y /= client_size.y;
  gl_Position = vec4(vertex_pos, 0.0, 1.0);
}
