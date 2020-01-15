#version 430 core

uniform mat4 uMatrixViewProjection;
uniform mat4 uMatrixView;
uniform mat4 uMatrixModel;

layout(location = 0) in vec3 inVertex;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) in vec3 inNormal;
//layout(location = 1) in uint inBarycentricIndex;
//layout(location = 2) in vec3 inColor;

//uniform mat4 uMatrixVP;
//in mat3 inModelMatrix;

out VVertexData {
	vec2 vModelTexCoord;
	vec3 vNormal;
	vec3 vTangent;
	vec3 vBitangent;
	vec3 vVertexWorldspace;
	vec3 vEyeDirWorldspace;
	vec3 vVertexColor;
};
//out vec3 tBaryCoord;
//out vec3 tVertexColor;

void main() {
	/*
	const vec3 BARYCENTRIC_COORD_TABLE[3] = vec3[3](
		vec3(1.0, 0.0, 0.0),
		vec3(0.0, 1.0, 0.0),
		vec3(0.0, 0.0, 1.0)
	);
	tBaryCoord = BARYCENTRIC_COORD_TABLE[inBarycentricIndex]
	*/
//	tBaryCoord = vec3(1.0);
	
//	tBaryCoord = vec3(0.0);
//	tVertexColor = vec3(1.0);
	
	vModelTexCoord = vec2(inTexCoord.s, 1.0 - inTexCoord.t);
	vec4 homogenousNormal = transpose(inverse(uMatrixModel)) * vec4(inNormal.xyz, 1.0);
	vNormal = homogenousNormal.xyz / homogenousNormal.w;
//	vVertexColor = vec3(0.0, 1.0, 0.5);
	
	//vec4 eyeDirHomogenous = transpose(inverse(uMatrixViewProjection)) * vec4(vec3(0.0, 0.0, 1.0), 1.0);
	vec4 worldspaceVertex = uMatrixModel * vec4(inVertex.xyz, 1.0);
	vVertexWorldspace = worldspaceVertex.xyz / worldspaceVertex.w;
	
	vec4 eyeDirHomogenous = inverse(uMatrixView) * vec4(0.0, 0.0, 0.0, 1.0);
	vEyeDirWorldspace = eyeDirHomogenous.xyz / eyeDirHomogenous.w;	
	
	gl_Position = uMatrixViewProjection * worldspaceVertex;
}
