#version 430 core

#define SEPARABLE_SSS_KERNEL_SIZE 7

#define FXAA_ENABLE false

layout(binding = 0) uniform sampler2D texSceneHDR;
layout(binding = 1) uniform sampler2D texEHAAEdgeHeuristic;

uniform vec4 uSeparableSSSKernel[SEPARABLE_SSS_KERNEL_SIZE] = {
	vec4(0.62899023, 0.75196564, 0.8388238, 0),
	vec4(0.0070557906, 0.00021022449, 0.000051764513, -2),
	vec4(0.047199063, 0.010689508, 0.004010269, -0.8888888),
	vec4(0.10899024, 0.16196562, 0.11882374, 0),
	vec4(0.15350983, 0.0642693, 0.03422845, 0.22222227),
	vec4(0.04719904, 0.010689498, 0.0040102643, 0.8888891),
	vec4(0.0070557883, 0.00021022448, 0.000051764502, 2),
};

in vec2 vScreenTexCoord;

out vec3 outFragProcessed;

/* passAxis = vec2(1.0, 0.0) (first pass), vec2(0.0, 1.0) (second pass) */
void seperablesssResolve(vec2 screenTexCoord, float sssWidth, vec2 passAxis) {
	
}

vec3 tonemapFrag(vec3 fragHDR) {
	vec3 mappedFrag = pow(fragHDR, vec3(1.0/2.2)); // Simple gamma correction
	return mappedFrag;
}

float ungammaLuma(vec3 rgb) {
	rgb = tonemapFrag(rgb);
	const vec3 A = vec3(0.299, 0.587, 0.114);
	return sqrt(dot(rgb, A));
}

const float FXAA_EDGE_THRESHOLD_MIN = 0.0312;
const float FXAA_EDGE_THRESHOLD_MAX = 0.1250;

vec3 resolveFXAA(vec3 inputFrag, vec2 screenUV, sampler2D sceneTex) {
	// http://blog.simonrodriguez.fr/articles/30-07-2016_implementing_fxaa.html
	
	vec2 inverseScreenSize = 1.0 / vec2(textureSize(sceneTex, 0)); 
	
	// Get luma for current frag
	float lumaCenter = ungammaLuma(inputFrag);
	
	// Sample neighbor scene frags
	float lumaDown = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(0, -1)).rgb);
	float lumaUp = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(0, 1)).rgb);
	float lumaLeft = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(-1, 0)).rgb);
	float lumaRight = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(1, 0)).rgb);
	
	// Find the maximum and minimum luma around our frag
	float lumaMin = min(lumaCenter, min(min(lumaDown, lumaUp), min(lumaRight, lumaLeft)));
	float lumaMax = max(lumaCenter, max(max(lumaDown, lumaUp), max(lumaRight, lumaLeft)));
	
	// Compute the delta
	float lumaDelta = lumaMax - lumaMin;
	
	// Check if in threshold
	if(lumaDelta < max(FXAA_EDGE_THRESHOLD_MIN, lumaMax * FXAA_EDGE_THRESHOLD_MAX)) {
		return inputFrag;
	}
	
	// Query the 4 remaining corners
	float lumaDownLeft = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(-1, -1)).rgb);
	float lumaUpRight = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(1, 1)).rgb);
	float lumaUpLeft = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(-1, 1)).rgb);
	float lumaDownRight = ungammaLuma(textureOffset(sceneTex, screenUV, ivec2(1, -1)).rgb);
	
	// Combine the four edge lumas
	float lumaDownUp = lumaDown + lumaUp;
	float lumaLeftRight = lumaLeft + lumaRight;
	
	// Same the corners
	float lumaLeftCorners = lumaDownLeft + lumaUpLeft;
	float lumaDownCorners = lumaDownLeft + lumaDownRight;
	float lumaRightCorners = lumaDownRight + lumaUpRight;
	float lumaUpCorners = lumaUpRight + lumaUpLeft;
	
	// Compute an estimation of the gradient along the horizontal and vertical axes
	float edgeHorizontal = abs(-2.0 * lumaLeft + lumaLeftCorners) + 2.0 * abs(-2.0 * lumaCenter + lumaDownUp) + abs(-2.0 * lumaRight + lumaRightCorners);
	float edgeVertical = abs(-2.0 * lumaUp + lumaUpCorners) + 2.0 * abs(-2.0 * lumaCenter + lumaLeftRight) + abs(-2.0 * lumaDown + lumaDownCorners);
	
	bool isHorizontal = (edgeHorizontal >= edgeVertical);
	
	// Select the two neighboring texels lumas in the opposite direction to the local edge
	float luma1 = isHorizontal ? lumaDown : lumaLeft;
	float luma2 = isHorizontal ? lumaUp : lumaRight;
	
	// Compute gradients
	float gradient1 = luma1 - lumaCenter;
	float gradient2 = luma2 - lumaCenter;
	
	// Check which direction is the steepest
	bool is1Steepest = abs(gradient1) >= abs(gradient2);
	
	// Gradient in the corresponding direction
	float gradientScaled = 0.25 * max(abs(gradient1), abs(gradient2));
	
	// Choose the step size (one pixel) according to the edge direction
	float stepLength = isHorizontal ? inverseScreenSize.y : inverseScreenSize.x;
	
	// Average luma in the correct direction
	float lumaLocalAverage = 0.0;
	
	if(is1Steepest) {
		// Switch the direction
		stepLength = -stepLength;
		lumaLocalAverage = 0.5 * (luma1 + lumaCenter);
	}
	else {
		lumaLocalAverage = 0.5 * (luma2 + lumaCenter);
	}
	
	// Shift uv in the correct direction by half a pixel
	vec2 currentUV = screenUV;
	if(isHorizontal) {
		currentUV.y += stepLength * 0.5;
	}
	else {
		currentUV.x += stepLength * 0.5;
	}
	
	// Compute offset (for each iteration step) in the right direction
	vec2 offset = isHorizontal ? vec2(inverseScreenSize.x, 0.0) : vec2(0.0, inverseScreenSize.y);
	
	// Compute uvs to explore on each side of the edge, orthogonally
	vec2 uv1 = currentUV - offset;
	vec2 uv2 = currentUV + offset;
	
	// Read the lumas at both the current extremities of the exploration target, and comptue the delta wrt. the local average luma
	float lumaEnd1 = ungammaLuma(texture(sceneTex, uv1).rgb);
	float lumaEnd2 = ungammaLuma(texture(sceneTex, uv2).rgb);
	lumaEnd1 -= lumaLocalAverage;
	lumaEnd2 -= lumaLocalAverage;
	
	// If the luma deltas at the current extremities are larger than the local gradient we have reached the side of the edge
	bool reached1 = abs(lumaEnd1) >= gradientScaled;
	bool reached2 = abs(lumaEnd2) >= gradientScaled;
	bool reachedBoth = reached1 && reached2;
	
	// If the side is not reached, we continue to explore in this direction
	if(!reached1) {
		uv1 -= offset;
	}
	if(!reached2) {
		uv2 += offset;
	}
	
	// If both sides have no tbeen reached, continue to explore
	if(!reachedBoth) {
		const uint ITERATIONS = 12;
		
		for(uint i = 2; i < ITERATIONS; i++) {
			// If needed, read luma in 1st direction and compute delta
			if(!reached1) {
				lumaEnd1 = ungammaLuma(texture(sceneTex, uv1).rgb);
				lumaEnd1 = lumaEnd1 - lumaLocalAverage;
			}
			
			// If needed, read luma in opposite direction and compute delta
			if(!reached2) {
				lumaEnd2 = ungammaLuma(texture(sceneTex, uv2).rgb);
				lumaEnd2 = lumaEnd2 - lumaLocalAverage;
			}
			
			// If the luma deltas at the current extremitie are larger than the local gradient we have reached the side of the edge
			reached1 = abs(lumaEnd1) >= gradientScaled;
			reached2 = abs(lumaEnd2) >= gradientScaled;
			reachedBoth = reached1 && reached2;
			
//			const uint NUM_QUALITY_SAMPLES = 12;
//			const float QUALITY_SAMPLES[ITERATIONS] = float[ITERATIONS](1.5, 2.0, 2.0, 2.0, 2.0, 4.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0);
			
			// If the side is not reached we continue to explore in this direction with a variable quality
			if(!reached1) {
//				uv1 -= offset * QUALITY_SAMPLES[i];
				uv1 -= offset * 1.0;
			}
			if(!reached2) {
//				uv2 += offset * QUALITY_SAMPLES[i];
				uv2 += offset * 1.0;
			}
			
			// If both sides have been reached, stop the exploration
			if(reachedBoth) {
				break;
			}
		}
		
		// Compute the distances to each extremity of the edge
		float distance1 = isHorizontal ? (screenUV.x - uv1.x) : (screenUV.y - uv1.y);
		float distance2 = isHorizontal ? (uv2.x - screenUV.x) : (uv2.y - screenUV.y);
		
		// Check in which direction the extremity of the edge is closer
		bool isDirection1 = distance1 < distance2;
		float distanceFinal = min(distance1, distance2);
		
		// Length of the edge
		float edgeThickness = (distance1 + distance2);
		
		// UV offset: Read in the direction of the closest side of the edge
		float pixelOffset = -distanceFinal / edgeThickness + 0.5;
		
		// Check if the luma at the center is smaller than the local average
		bool isLumaCenterSmaller = lumaCenter < lumaLocalAverage;
		
		// If the luma at center is smaller thatn it's neighbor the delta luma at each end should be positive (same variation)
		// (in the direction of the closer side of the edge)
		bool correctedVariation = ((isDirection1 ? lumaEnd1 : luma2) < 0.0) != isLumaCenterSmaller;
		
		// If the luma variation is incorrect, do not offset
		float finalOffset = correctedVariation ? pixelOffset : 0.0;
		
		// Subpixel shifting
		float lumaAverage = (1.0 / 12.0) * (2.0 * (lumaDownUp + lumaLeftRight) + lumaLeftCorners + lumaRightCorners);
		
		// Ratio of the delta between the global average and the center luma over the luma range in the 3x3 neighborhood
		float subpixelOffset1 = clamp(abs(lumaAverage - lumaCenter) / lumaDelta, 0.0, 1.0);
		float subpixelOffset2 = (-2.0 * subpixelOffset1 + 3.0) * subpixelOffset1 * subpixelOffset1;
		
		// Compute a subpixel offset based on the delta
		const float SUBPIXEL_QUALITY = 0.75;
		float subpixelOffsetFinal = subpixelOffset2 * subpixelOffset2 * SUBPIXEL_QUALITY;
		
		// Pick the biggest of the two offsets
		finalOffset = max(finalOffset, subpixelOffsetFinal);
		
		// Compute the final uv coords
		vec2 finalUV = screenUV;
		if(isHorizontal) {
			finalUV.y += finalOffset * stepLength;
		}
		else {
			finalUV.x += finalOffset * stepLength;
		}
		
		// Read the color at the new uv coord and use it
		vec3 finalFXAAFrag = texture(sceneTex, finalUV).rgb;
		return finalFXAAFrag;
	}
}

void main() {
	// Sample scene frag
	vec3 sceneFragHDR = texture(texSceneHDR, vScreenTexCoord.st).rgb;
	
	{// New algorithm
		// PROBLEM #1: We can still sample our own triangle by accident on slight angles
		// PROBLEM #2: How to blend pulled samples together? (Possibly related to the problem #1)
		
		const vec2 invTexSize = 1.0 / vec2(textureSize(texSceneHDR, 0));
		vec4 edgeCoeffs = texture(texEHAAEdgeHeuristic, vScreenTexCoord.st).xyzw;
		
		vec4 adjacentCoverageFactors = normalize(edgeCoeffs.xyzw);
		
		float maxCoeff = max(max(edgeCoeffs[0], edgeCoeffs[1]), max(edgeCoeffs[2], edgeCoeffs[3]));
		if(maxCoeff > 0.0001) {
			vec3 pulledFrag = vec3(0.0);
			pulledFrag += texture(texSceneHDR, vScreenTexCoord + vec2(invTexSize.x, 0.0)).rgb * adjacentCoverageFactors[0];
			pulledFrag += texture(texSceneHDR, vScreenTexCoord + vec2(0.0, invTexSize.y)).rgb * adjacentCoverageFactors[1];
			pulledFrag += texture(texSceneHDR, vScreenTexCoord - vec2(invTexSize.x, 0.0)).rgb * adjacentCoverageFactors[2];
			pulledFrag += texture(texSceneHDR, vScreenTexCoord - vec2(0.0, invTexSize.y)).rgb * adjacentCoverageFactors[3];
			
	//		sceneFragHDR = mix(sceneFragHDR, pulledFrag, maxCoeff);
		}
		
		// DEBUG:
//		frag = vec3(edgeCoeffs.xyz);
	}
	
	/*
	{// DEBUG:
		vec2 invTexSize = 1.0 / vec2(textureSize(texSceneHDR, 0));
		
		vec4 ehaaFrag = texture(texEHAAEdgeHeuristic, vScreenTexCoord.st).xyzw;
		vec2 normal = normalize(ehaaFrag.yz * 2.0 - 1.0);
		
		vec2 sampleNormal = sign(normal) * step(vec2(0.5), abs(normal));
		
		vec2 sampleOffset = invTexSize * sampleNormal;
		vec3 pulledFrag = texture(texSceneHDR, vScreenTexCoord + sampleOffset).rgb;
		
		sceneFragHDR = mix(tonemapFrag(sceneFragHDR), tonemapFrag(pulledFrag), ehaaFrag.x);
		//sceneFragHDR = vec3(sampleNormal * 0.5 + 0.5, 0.0);
	}
	*/
	
	{// Resolve separable sss
	}
	
	// Tone map and gamma correct fragment
	vec3 tonemappedFrag;
	
	// Do fxaa
	#if FXAA_ENABLE
		tonemappedFrag = tonemapFrag(resolveFXAA(sceneFragHDR, vScreenTexCoord.st, texSceneHDR));
	#else
		tonemappedFrag = tonemapFrag(sceneFragHDR);
//		tonemappedFrag = sceneFragHDR;
	#endif
	
	// Write final processed frag
	outFragProcessed = tonemappedFrag;
}
