#version 430 core

//layout(location = 0) in vec3 inVertex;
//layout(location = 1) in vec2 inTexCoord;

out vec2 vTexCoord;

void main() {
	vec2 xy = -1.0 + vec2(float((gl_VertexID & 1) << 2), float((gl_VertexID & 2) << 1));
	vTexCoord = xy * 0.5 + 0.5;
	gl_Position = vec4(xy, 0.0, 1.0);
}
