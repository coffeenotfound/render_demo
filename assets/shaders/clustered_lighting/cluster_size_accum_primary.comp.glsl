#version 430 core

// The first parallel prefix-sum pass for accumulating the
// sizes (number of light indices / light data pages) per cluster.
// This is needed so we can later dissolve the light assignment
// linked list fully parallel into the final cluster light buffer.

// Do 1024 clusters in parallel.
// (This is the minimum supported size and the max of most modern GPUs)
layout(local_size_x = 1024, local_size_y = 1, local_size_z = 1) in;

layout(std140) readonly restrict ssboClusterSizeBuffer {
	
};

shared uint sharedAccumClusterSizeArray[1024];

void main() {
	
}
