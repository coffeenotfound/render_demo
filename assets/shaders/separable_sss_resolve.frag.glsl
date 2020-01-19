#version 430 core

#define SSSS_FOLLOW_SURFACE 1

#define SSSS_NUM_SAMPLES 25
vec4 SSSS_KERNEL[] = {
	vec4(0.530605, 0.613514, 0.739601, 0.0),
	vec4(0.000973794, 1.11862e-005, 9.43437e-007, -3.0),
	vec4(0.00333804, 7.85443e-005, 1.2945e-005, -2.52083),
	vec4(0.00500364, 0.00020094, 5.28848e-005, -2.08333),
	vec4(0.00700976, 0.00049366, 0.000151938, -1.6875),
	vec4(0.0094389, 0.00139119, 0.000416598, -1.33333),
	vec4(0.0128496, 0.00356329, 0.00132016, -1.02083),
	vec4(0.017924, 0.00711691, 0.00347194, -0.75),
	vec4(0.0263642, 0.0119715, 0.00684598, -0.520833),
	vec4(0.0410172, 0.0199899, 0.0118481, -0.333333),
	vec4(0.0493588, 0.0367726, 0.0219485, -0.1875),
	vec4(0.0402784, 0.0657244, 0.04631, -0.0833333),
	vec4(0.0211412, 0.0459286, 0.0378196, -0.0208333),
	vec4(0.0211412, 0.0459286, 0.0378196, 0.0208333),
	vec4(0.0402784, 0.0657244, 0.04631, 0.0833333),
	vec4(0.0493588, 0.0367726, 0.0219485, 0.1875),
	vec4(0.0410172, 0.0199899, 0.0118481, 0.333333),
	vec4(0.0263642, 0.0119715, 0.00684598, 0.520833),
	vec4(0.017924, 0.00711691, 0.00347194, 0.75),
	vec4(0.0128496, 0.00356329, 0.00132016, 1.02083),
	vec4(0.0094389, 0.00139119, 0.000416598, 1.33333),
	vec4(0.00700976, 0.00049366, 0.000151938, 1.6875),
	vec4(0.00500364, 0.00020094, 5.28848e-005, 2.08333),
	vec4(0.00333804, 7.85443e-005, 1.2945e-005, 2.52083),
	vec4(0.000973794, 1.11862e-005, 9.43437e-007, 3.0),
};

/**
 * For the first pass the final scene hdr render target,
 * for the second pass the intermediate rt from the first
 * sss resolve pass.
 */
layout(binding = 0) uniform sampler2D texSceneHDR;

/**
 * Depth buffer of the scene. The depth should be linear but we
 * are using reverse float z which is linear-ish enough
 */
layout(binding = 1) uniform sampler2D texSceneDepth;

/** Global width of the seperable sss filter in worldspace units */
//uniform float uGlobalSSSWidth = 0.012;
uniform float uGlobalSSSWidth = 0.025;

/** Direction of the filter pass. vec2(1.0, 0.0) for first pass, vec2(0.0, 1.0) for second. */
uniform vec2 uSeperablePassDir;
uniform float uCameraFovyRad;

/**
 * Contains the camera's vec2(nearZ, farZ).
 * Note that nearZ is always smaller than farZ, no matter
 * if we're using reverse depth or not.
 */
uniform vec2 uCameraDepthPlanes;

in vec2 vScreenTexCoord;

out vec3 outBlurredFragHDR;

float linearizeDepth(float logarithmicDepth, vec2 depthPlanes) {
	float near = depthPlanes.x;
	float far = depthPlanes.y;
	return (2.0 * near) / (far + near - (logarithmicDepth * (far - near)));
}

void main() {
	vec2 texCoord = vScreenTexCoord.st;
	
	// Sample the current frag
	vec3 frag = texture(texSceneHDR, texCoord.st).rgb;
	float linearDepth = linearizeDepth(1.0 - texture(texSceneDepth, texCoord.st).r, uCameraDepthPlanes);
	
	/*
	// DEBUG:
	if(linearDepth > 0.9) {
		outBlurredFragHDR = frag;
		return;
	}
	*/
	
	float sampledSubsurfaceStrength = 0.83; // TODO: Sample from a rendertarget
	
	// Calculate the screenspace filter scale (1.0 for a unit plane sitting on the projection window)
	float distanceToProjectionWindow = 1.0 / tan(/*0.5 * */ uCameraFovyRad); // TODO: Calc this on the cpu and send through a uniform
	float scale = distanceToProjectionWindow / linearDepth;
	
	// Calculate the final step to fetch the other samples
	vec2 finalStep = scale * uGlobalSSSWidth * uSeperablePassDir;
	finalStep *= sampledSubsurfaceStrength;
	finalStep *= 1.0 / 3.0; // Divide by 3 as the kernel's range is [-3, 3]
	
	finalStep *= 1.0 / vec2(textureSize(texSceneHDR, 0));
	
	// Accumulate the center sample
	vec3 blurredFrag = frag * SSSS_KERNEL[0].rgb;
	
	// Apply the filter
	for(int i = 1; i < SSSS_NUM_SAMPLES; i++) {
		vec4 kernelSample = SSSS_KERNEL[i];
		
		// Fetch sample data
		vec2 offsetTexCoord = texCoord + finalStep * kernelSample.a;
		vec3 sampleFrag = texture(texSceneHDR, offsetTexCoord.st).rgb;
		
		#if SSSS_FOLLOW_SURFACE == 1
			float sampleLinearDepth = linearizeDepth(1.0 - texture(texSceneDepth, offsetTexCoord.st).r, uCameraDepthPlanes);
			
			// If the difference in depth is huge, we lerp color back to the center frag color
			//float s = clamp(300.0 * distanceToProjectionWindow * uGlobalSSSWidth * abs(linearDepth - sampleLinearDepth), 0.0, 1.0);
			//sampleFrag = mix(sampleFrag, frag, s);
			
			float s = clamp(2000000.0 * distanceToProjectionWindow * uGlobalSSSWidth * abs(linearDepth - sampleLinearDepth), 0.0, 1.0);
			sampleFrag = mix(sampleFrag, frag, s);
		#endif
		
		// Accumulate
		blurredFrag += kernelSample.rgb * sampleFrag;
	}
	
	// Write out blurred frag
	outBlurredFragHDR = blurredFrag;
}
