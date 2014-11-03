BISMIT

Biologically Inspired Sensory Motor Inference Tool

A model of the neocortex for interpreting and classifying (input) data for the purpose of taking action (output).

A descendant of Bayesian and hidden Markov neural networks and incorporating concepts from hierarchical temporal memory, Bismit compartmentalizes the task of inference into sections, referred to as columns. These columns have layers, each which interacts only with other cells (neurons) of the same layer from nearby columns. The task of each cell within a column is to infer future input and use avaliable information to direct action. It does this both by feeding information "upwards" to higher tiers of cortex, and by using local information to feed actionable information "downwards," to lower tiers.

All the usual suspects are involved including systems for inhibition, sequence, ...(tbc) 
