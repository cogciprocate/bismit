#BISMIT

Biologically Inspired Sensory Motor Inference Tool: 
A model of the neocortex for learning and taking action.

Bismit is one of the first members of the next paradigm of cortical learning networks. Going beyond simple Bayesian neural networks and incorporating ideas from the theory of hierarchical temporal memory, such as sparse distributed representations and temporal context, Bismit is a model which incorporates our most recent and up-to-date findings about the neocortex. It is not a typical "Machine Learning" platform and does not use traditional statistical methods (though they are a fundamental part of its theory). 

Intended to be used as a platform for prototyping and testing completely different arrangements of both connections between cortical areas (regions) and layer structure within a cortical area.

Bismit uses a structure which mirrors the human neocortex very closely. The cortex is broken into 'areas', which are broken into 'layers', and so on. Here is a full hierarchy:
	- Cortex
		- Areas
			- Layers
				- Slices
					- Cells
						- Cell tufts (in the case of pyramidal cells)
							- Dendrites
								- Synapses

The cortex is also granulated into columns in the other two dimensions, exactly like the real neocortex.

Much more information and documentation coming soon.

Bismit is written in Rust and OpenCL C and is in an unstable pre-alpha stage. Full basic functionality is expected by the end 2015.



[//]: # (POSTSCRIPT (move this elsewhere): )

[//]: # (We are now within close reach, eight to ten years or a factor of about a thousand, of matching the processing power of the human neocortex on a $1000 device. By harnessing the inexpensive parallel computing power offered by GPGPUs (or any other OpenCL device) we can build a platform to best utilize that amount of power, intended for maximum performance with maximum flexibility in computing device interoperability.)
