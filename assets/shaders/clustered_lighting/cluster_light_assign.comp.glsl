#version 430 core

#define CURRENT_LIGHT_TYPE_POINT_LIGHT 0
#define CURRENT_LIGHT_TYPE_SPOT_LIGHT 1
#define CURRENT_LIGHT_TYPE_AREA_LIGHT 2

#define NUM_CLUSTERS_XYZ uvec3(64, 64, 16)
#define NUM_CLUSTERS_TOTAL (NUM_CLUSTERS_XYZ.x * NUM_CLUSTERS_XYZ.y * NUM_CLUSTERS_XYZ.z)

#define SSBO_POINT_LIGHT_DATA_BINDING 4
#define SSBO_SPOT_LIGHT_DATA_BINDING 5
#define SSBO_AREA_LIGHT_DATA_BINDING 6

uniform uint uNumPointLights;
uniform uint uNumSpotLights;
uniform uint uNumAreaLights;

uniform uint uCurrentPassLightType;

struct PointLightData {
	vec4 positionAndNothing;
	vec4 intensitiesAndNothing;
};

struct SpotLightData {
	vec4 positionAndNothing;
	vec4 intensitiesAndInnerAngle;
	vec4 directionAndOuterAngle;
	vec4 other;
};

struct AreaLightData {
	vec4 centerAndPlaneWidth;
	vec4 intensitiesAndPlaneHeight;
	vec4 normalAndOuterAngle;
	vec4 tangentAndCookieIndex;
};

layout(std430, binding = SSBO_POINT_LIGHT_DATA_BINDING)
readonly restrict ssboPointLightData {
	PointLightData data[];
};

layout(std430, binding = SSBO_SPOT_LIGHT_DATA_BINDING)
readonly restrict ssboSpotLightData {
	SpotLightData data[];
};

layout(std430, binding = SSBO_AREA_LIGHT_DATA_BINDING)
readonly restrict ssboAreaLightData {
	AreaLightData data[];
};

struct ClusterNumberStruct {
	uvec4 lightNumbers;
	
	/** Must be 0xFFFFFFFF at the start */
	uint localAssignmentListHead;
	uint _padding0;
};

layout(std430, binding = 1) restrict buffer ssboClusterNumberBuffer {
	ClusterNumberStruct data[NUM_CLUSTERS_TOTAL];
};

struct AssignmentListEntry {
	uint lightDataIndex;
	uint nextEntryIndex;
};

layout(std430, binding = 2) restrict buffer ssboClusterAssignmentList {
	AssignmentListEntry entries[];
};

layout(std430, binding = 3) restrict buffer ssboClusterAssignWorkAtomics {
	uint assignmentListHead = 0;
};

void main() {
	if(uCurrentPassLightType == CURRENT_LIGHT_TYPE_POINT_LIGHT) {
		uint currentLightIndex = 0;
		
		// Do frustum culling
		bool isInFrustum = false;
		
		if(isInFrustum) {
			// Iterate over covered clusters
			for(int z) {
				for(int y) {
					for(int x) {
						uint clusterIndex = x + y*NUM_CLUSTERS_XYZ.x + z*NUM_CLUSTERS_XYZ.y;
						
						// Claim the next assignment list entry
						uint claimedEntryIndex = atomicAdd(ssboClusterAssignWorkAtomics.assignmentListHead, 1);
						uint lastEntry = atomicExchange(ssboClusterNumberBuffer.data[clusterIndex].localAssignmentListHead, claimedEntryIndex);
						
						atomicAdd(ssboClusterNumberBuffer.data[clusterIndex].lightNumbers.x, 1);
						
						if(lastEntry != 0xFFFFFFFF) {
							ssboClusterAssignmentList.entries[lastEntry].nextEntryIndex = claimedEntryIndex;
						}
						ssboClusterAssignmentList.entries[claimedEntryIndex].lightDataIndex = currentLightIndex;
					}
				}
			}
		}
	}
	else if {
		
	}
}
