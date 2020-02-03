#version 430 core

layout(binding = 0) uniform sampler2D texSceneHDR;
layout(binding = 1) uniform sampler2D texEHAAEdgeHeuristic; // GL_RGBA8 // GL_RG16_SNORM

in vec2 vTexCoord;

layout(location = 0) out vec3 outFrag;

void main() {
	// Sample scene rt
	vec3 frag = texture(texSceneHDR, vTexCoord.st).rgb;
	
	{// New algorithm
		// PROBLEM #1: We can still sample our own triangle by accident on slight angles
		// PROBLEM #2: How to blend pulled samples together? (Possibly related to the problem #1)
		
		vec4 edgeCoeffs = texture(texEHAAEdgeHeuristic, vTexCoord.st).xyzw;
		
		vec2 invTexSize = 1.0 / vec2(textureSize(texSceneHDR, 0));
		
		vec4 adjacentCoverageFactors = edgeCoeffs.xyzw / max(0.00000001, length(edgeCoeffs.xyzw));
		vec3 pulledFrag = vec3(0.0);
		pulledFrag += texture(texSceneHDR, vTexCoord + vec2(invTexSize.x, 0.0)).rgb * adjacentCoverageFactors[0];
		pulledFrag += texture(texSceneHDR, vTexCoord + vec2(0.0, invTexSize.y)).rgb * adjacentCoverageFactors[1];
		pulledFrag += texture(texSceneHDR, vTexCoord - vec2(invTexSize.x, 0.0)).rgb * adjacentCoverageFactors[2];
		pulledFrag += texture(texSceneHDR, vTexCoord - vec2(0.0, invTexSize.y)).rgb * adjacentCoverageFactors[3];
//		pulledFrag += frag * (1.0 - dot(adjacentCoverageFactors, vec4(1.0)));
		
//		frag = mix(frag, pulledFrag, max(edgeCoeffs[0], max(edgeCoeffs[1], max(edgeCoeffs[2], edgeCoeffs[3]))));
		
		// DEBUG:
//		frag = vec3(edgeCoeffs.xyz);
	}
	
	/*
	// Sample edge heuristic
	vec2 edgeHeuristic = texture(texEHAAEdgeHeuristic, vTexCoord.st).xy;
//	frag = vec3(length(edgeHeuristic));
	float edgeLength = length(edgeHeuristic);
	vec2 edgeNormal = edgeHeuristic / edgeLength;
	
	float edgePullFactor = mix(0.5, 0.0, smoothstep(0.0, 0.5, 1.0 - edgeLength));
	vec2 pullTexCoord = vTexCoord - (edgeNormal / vec2(textureSize(texSceneHDR, 0)));
	
	vec3 pulledFrag = texture(texSceneHDR, pullTexCoord.st).rgb;
	frag = mix(frag, pulledFrag, edgePullFactor);
	*/
	
	/*
	// Do new edge heuristic
	vec4 edgeHeuristic = texture(texEHAAEdgeHeuristic, vTexCoord.st).xyzw;
	
	vec2 invTexSize = 1.0 / vec2(textureSize(texSceneHDR, 0));
	
	vec3 pulledFrags = vec3(0.0);
	float combinedAdjacentFactor = 0.0;
	
	{
		vec2 offsetUV = vTexCoord + vec2(invTexSize.x, 0.0);
		
		float adjacentCoverageFactor = smoothstep(0.5, 1.0, texture(texEHAAEdgeHeuristic, offsetUV)[2]);
		float selfCoverageFactor = smoothstep(0.5, 0.0, edgeHeuristic[0]);
		float sampleFactor = 0.5*(adjacentCoverageFactor + selfCoverageFactor) * 0.25;
		
		combinedAdjacentFactor += sampleFactor;
		pulledFrags += texture(texSceneHDR, offsetUV).rgb * sampleFactor;
	}
	
	frag = pulledFrags + frag * (1.0 - combinedAdjacentFactor);
	*/
	
//	pulledFrag += texture(texSceneHDR, vTexCoord + vec2(invTexSize.x, 0.0)).rgb * edgeHeuristic[0];
	
	/*
	// DEBUG:
	if(edgeLength > 0.0) {
		frag = mix(frag, vec3(1.0), smoothstep(0.5, 1.0, 1.0 - edgeLength));
	}
	*/
	
	// Write final ldr frag
	outFrag = frag;
//	outFrag = pulledFrag;
//	outFrag = edgeHeuristic.xyz;
}
