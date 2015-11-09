#Bismit

Biologically Inspired Sensory Motor Inference Tool: 
A model of the neocortex for learning and taking action.

Bismit is one of the first members of the next paradigm of cortical learning networks. Going beyond simple Bayesian neural networks and incorporating ideas from the theory of hierarchical temporal memory, such as sparse distributed representations and temporal context, Bismit is a model which incorporates our most recent and up-to-date findings about the neocortex. It is not a typical machine learning or deep learning platform and does not use traditional statistical methods (though bayesian probability forms the theoretical building blocks). 

Intended to be used as a platform for prototyping and testing completely different arrangements of both connections between cortical areas (regions) and layer structure within a cortical area.

Bismit uses a structure which maps to the human neocortex.  Following is a full hierarchy of the structures in use.
	- Cortex
		- Region (v, u, w)
			- Area (v, u, w)
				- Layer (z)
					- Slice (z)
						- Cell (z, v, u, w)
							- Tuft
								- Dendrite
									- Synapse

Coordinates listed are those relevant to each level: 'z' represents depth, v, u, and w represent the the remaning two dimensions (normally x and y) divided into the three dimensions required for hexagonal grids.

Much more information and documentation coming soon.

Bismit is written in Rust and OpenCL C and is in an unstable pre-alpha stage. Full basic functionality is expected by the end 2015.



[//]: # (POSTSCRIPT (move this elsewhere): )

[//]: # (We are now within close reach, eight to ten years or a factor of about a thousand, of matching the processing power of the human neocortex on a $1000 device. By harnessing the inexpensive parallel computing power offered by GPGPUs (or any other OpenCL device) we can build a platform to best utilize that amount of power, intended for maximum performance with maximum flexibility in computing device interoperability.)
