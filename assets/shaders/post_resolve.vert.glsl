#version 430 core

out vec2 vScreenTexCoord;

void main() {
	vec2 xy = -1.0 + vec2(float((gl_VertexID & 1) << 2), float((gl_VertexID & 2) << 1));
	vScreenTexCoord = xy * 0.5 + 0.5;
	
	gl_Position = vec4(xy, 0.0, 1.0);
}
