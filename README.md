#Bismit

Biologically Inspired Sensory Motor Inference Tool: 
A model of the neocortex for learning and taking action.

Bismit is one of the first members of the next paradigm of cortical learning networks. Going beyond simple Bayesian neural networks and incorporating ideas from the theory of hierarchical temporal memory, such as sparse distributed representations and temporal context, Bismit is a model which incorporates our most recent and up-to-date findings about the neocortex. It is not a typical machine learning or deep learning platform and does not function like a traditional neural network. 

Bismit uses a structure which maps to the human neocortex.  Following is a full hierarchy of the structures in use.
- Cortex
   - Region
      - Area
         - Layer
            - Slice
               - Cell
                  - Tuft
                     - Dendrite
                        - Synapse

Much more information and documentation coming.

Bismit is written in Rust and OpenCL C and is in an unstable pre-alpha stage. Full basic functionality is expected by the end 2015.



[//]: # (POSTSCRIPT (move this elsewhere): )

[//]: # (We are now within close reach, eight to ten years or a factor of about a thousand, of matching the processing power of the human neocortex on a $1000 device. By harnessing the inexpensive parallel computing power offered by GPGPUs (or any other OpenCL device) we can build a platform to best utilize that amount of power, intended for maximum performance with maximum flexibility in computing device interoperability.)
