#version 430 core
layout(triangles, ccw, equal_spacing) in;

in vec3 vVertexColor[];

in VVertexData {
	vec2 vModelTexCoord;
	vec3 vNormal;
	vec3 vTangent;
	vec3 vBitangent;
	vec3 vVertexWorldspace;
	vec3 vEyeDirWorldspace;
	vec3 vVertexColor;
//	noperspective vec3 vBaryCoord;
} vPerVertex[];

out vec2 tModelTexCoord;
out vec3 tNormal;
out vec3 tTangent;
out vec3 tBitangent;
out vec3 tVertexWorldspace;
out vec3 tEyeDirWorldspace;
noperspective out vec3 tBaryCoord;
flat out vec2 tBaryNormals[3];
flat out vec2 tBaryScreenCoords[3];
out vec3 tVertexColor;

flat out float tFuckOffPrimitiveIDFuckingShitDoesnWorkIndex;

vec3 baryInterp3(vec3 a, vec3 b, vec3 c, vec3 bary) {
	return a * bary.x + b * bary.y + c * bary.z;
}

vec2 baryInterp2(vec2 a, vec2 b, vec2 c, vec3 bary) {
	return a * bary.x + b * bary.y + c * bary.z;
}

vec3 makeTangent(vec3 A, vec3 B, vec3 C, vec2 Auv, vec2 Buv, vec2 Cuv) {
	float Bv_Cv = Buv.y - Cuv.y;
	if(Bv_Cv == 0.0) {
		return (B - C) / (Buv.x-Cuv.x);
	}
	
	float quotient = (Auv.y - Cuv.y) / (Bv_Cv);
	vec3 D = C + (B - C) * quotient;
	vec2 Duv = Cuv + (Buv-Cuv) * quotient;
	return (D - A) / (Duv.x - Auv.x);
}

vec3 makeBitangent(vec3 A, vec3 B, vec3 C,  vec2 Auv, vec2 Buv, vec2 Cuv) {
	return makeTangent(A, C, B, Auv.yx, Cuv.yx, Buv.yx);
}

void main() {
	/*
	uint vertexIndex = uint(dot(gl_TessCoord.xyz, vec3(1.0, 2.0, 3.0)));
	tBaryCoord = vBaryCoord[vertexIndex];
	tVertexColor = vVertexColor[vertexIndex];
	*/
	
	tBaryCoord = gl_TessCoord.xyz;
//	tVertexColor = baryInterp3(vPerVertex[0].vVertexColor, vPerVertex[1].vVertexColor, vPerVertex[2].vVertexColor, gl_TessCoord.xyz);
	tVertexColor = gl_TessCoord.xyz;
	tModelTexCoord = baryInterp2(vPerVertex[0].vModelTexCoord, vPerVertex[1].vModelTexCoord, vPerVertex[2].vModelTexCoord, gl_TessCoord.xyz);
	
	tNormal = baryInterp3(vPerVertex[0].vNormal, vPerVertex[1].vNormal, vPerVertex[2].vNormal, gl_TessCoord.xyz);
	tVertexWorldspace = baryInterp3(vPerVertex[0].vVertexWorldspace, vPerVertex[1].vVertexWorldspace, vPerVertex[2].vVertexWorldspace, gl_TessCoord.xyz);
	tEyeDirWorldspace = baryInterp3(vPerVertex[0].vEyeDirWorldspace, vPerVertex[1].vEyeDirWorldspace, vPerVertex[2].vEyeDirWorldspace, gl_TessCoord.xyz);
	
	//tTangent = makeTangent(vPerVertex[0].vPositionWorldSpace, vPerVertex[1].vPositionWorldSpace, vPerVertex[2].vPositionWorldSpace, vPerVertex[0].vModelTexCoord, vPerVertex[1].vModelTexCoord, vPerVertex[2].vModelTexCoord);
	//tBitangent = makeBitangent(vPerVertex[0].vPositionWorldSpace, vPerVertex[1].vPositionWorldSpace, vPerVertex[2].vPositionWorldSpace, vPerVertex[0].vModelTexCoord, vPerVertex[1].vModelTexCoord, vPerVertex[2].vModelTexCoord);
	
	tFuckOffPrimitiveIDFuckingShitDoesnWorkIndex = mod((gl_in[0].gl_Position.x - gl_in[2].gl_Position.x + gl_in[1].gl_Position.y) * 64.0, 64.0) / 64.0;
	
	// vec2(-1.0, 1.0) for CCW (and only CCW!!) winding order so the normal points outwards
	tBaryNormals[0] = vec2(-1.0, -1.0) * normalize(gl_in[1].gl_Position.xy - gl_in[2].gl_Position.xy).yx;
	tBaryNormals[1] = vec2(-1.0, -1.0) * normalize(gl_in[2].gl_Position.xy - gl_in[0].gl_Position.xy).yx;
	tBaryNormals[2] = vec2(-1.0, -1.0) * normalize(gl_in[0].gl_Position.xy - gl_in[1].gl_Position.xy).xy;
	
	vec2 screenSize = vec2(1280.0, 720.0);
	tBaryScreenCoords[0] = ((gl_in[0].gl_Position.xy / gl_in[0].gl_Position.w) * 0.5 + 0.5) * screenSize;
	tBaryScreenCoords[1] = ((gl_in[1].gl_Position.xy / gl_in[1].gl_Position.w) * 0.5 + 0.5) * screenSize;
	tBaryScreenCoords[2] = ((gl_in[2].gl_Position.xy / gl_in[2].gl_Position.w) * 0.5 + 0.5) * screenSize;
	
//	gl_Position = gl_in[vertexIndex].gl_Position;
	gl_Position = gl_in[0].gl_Position * gl_TessCoord[0] + gl_in[1].gl_Position * gl_TessCoord[1] + gl_in[2].gl_Position * gl_TessCoord[2];
}
