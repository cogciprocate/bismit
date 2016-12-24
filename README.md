## Bismit

##### Biologically Inspired Sensory Motor Inference Toolkit

Bismit is one of the first members of the next paradigm of cortical learning
libraries. Going beyond simple Bayesian neural networks and incorporating
ideas from the theory of [hierarchical temporal
memory](https://en.wikipedia.org/wiki/Hierarchical_temporal_memory), such as
sparse distributed representations and temporal context, Bismit is a brain
building framework for designing truly intelligent machines. It is not a
typical machine learning or deep learning platform and does not function like
a traditional neural network.


### Goals

To create:

* A toolkit that is simple enough to be useful now and be flexible enough to
  absorb future neuroscientific discoveries. We know enough now about the
  basic architecture of the brain that we can create a scaffold with enough
  wiggle room to be refined as more is learned.


### Structure

Bismit simulates the interactions of the following hierarchical structures of
the neocortex:

- Cortex
   - Region
      - Area
         - Layer
            - Slice
               - Cell
                  - Tuft
                     - Dendrite
                        - Synapse


Bismit is written in Rust and OpenCL C and is in an unstable pre-alpha stage.
Full basic sensory functionality is complete. Motor control and use case
development are underway. See the [vibi
project](https://github.com/cogciprocate/vibi) (currently in early
development) for an OpenGL based visualization frontend.

