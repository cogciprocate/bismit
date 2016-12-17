## The Pyramidal Cell is the Universal ANDer

Pyramidal cells are the fundamental basic processing unit for all complex logic in the brain. They are all one needs to describe mathematics, object detection, music, etc. By signaling coincidence between both known basal and known apical patterns, they act as a trainable AND gate which can be wired to signal on any conceivable combination of patterns. The output from the axon can be used directly as motor signal or combined with other layers to represent any other arbitrary signal or sequence of signals.


## Layer 5 TODO
- Wire up an output decoder (`Arm`)
- Implement random firing algorithm
    - Include dynamically adjustable frequency and add init val param to `LayerMapScheme`
- 


## Executor / Execution Context / Functional Context

- Represents the current desire
- Maps to an unconscious thought or thread
- Many, many run in parallel in a 'tangled tree' shaped heirarchy and compete
  for precedence
- Roughly maps to our abstract concept of a theory, frame of mind,
  perspective, approach, etc.
- Also related to our more pragmatic concept of a decision, course of action,
  prescription, game plan, etc.
- Handles the high level switching context necessary to carry out a sequence
  of actions given a certain stimulus

- Layer 6 (tall and short) is the key layer involved in this process.
- The Claustrum is likely a key structure involved in coordinating and
  broadcasting execution context information
- The nRT (and L6-short) is related to and intertwined with this system (and
  may have been a precursor system) but seems to focus on a different aspect
  (particularly timing-sync and sleep/power saving).


## Learning for pyramidal cell synapses (prediction oriented synapses)
    
Conceptual overview from an associational layer containing pyramidal cells (such as L3):

    Input and Activation (single column, single layer perspective):

        0.00t: Elsewhere in nearby layers (such as L4), data is received from the thalamus or other areas of the cortex

        0.10t: Input to our layer-column arrives at pyramidal cell (LCPC) proximal dendrites

        0.20t: If any pyramidal cell (LCPC) is depolarized as a result of prior distal dendritic activity and receives a suprathreshold amount of EPSP on its proximal dendrite (from prior time step), it will generate an EPSP ->

        0.30t: If the layer-columnar aspiny stellate inhibitory cell (LCASIC) receives an action potential from any pyramidal cell in our layer-column, it will generate an IPSP ->
        
        0.40t: If the layer-columnar pyramidal cells (LCPCs) do not receive an IPSP from the LCASIC and have received an EPSP (from 0.1t), they will each generate an EPSP
    
    Learning (single pyramidal cell perspective):


        ### NEW STUFF COMING ###

        ### MANY CHANGES COMING ###


        NOTE: Keep in mind that distal synaptic states at this instant still reflect information from the time period ~ 1.0t in the past. That is, they still contain information from one cycle ago. They do not get refreshed with information from the current cycle until just after learning takes place, for good reason.

        0.51t: If the pyramidal cell (LCPC) has generated an action potential:
            - The most active distal dendrite (keeping in mind this will still reflect data from the prior cycle) will be selected
            - Each synapse on this dendrite will be processed in the following manner:
                - If the synapse is active, apply a short-term positive potentiation to it
                - If the synapse is inactive, apply a long-term negative potentiation to it

        0.52t: If the LCPC has not generated an action potential AND the LCPC was active in the prior cycle:
            - NOTE: Somewhat confusing is the fact that in this step we are dealing with the the learning state of dendrites and synapses from the prior cycle, which itself is based on activation from the cycle prior to that. So basically we're now doing a follow-up and conclusion to the short-term learning applied in the step above (0.51t).

            - For each synapse on the most active distal dendrite from the prior cycle:
                - If the synapse had short-term potentiation applied:
                    - If the synapse is still active, apply a long-term negative potentiation to it
                    - If the synapse is now inactive, apply a long-term positive potentiation to it
                - Clear away all short-term potentiation

                - NOTE: The reason we potentiate the synapses which were recently active (and short-term potentiated) but are now inactive is that they are likely to be correctly predicting the input. Put another way, synapses which were active when the input was active and inactive now that the input is inactive are probably, or at least possibly, representing it. On the other hand, synapses which were active when the input was active and remain active now that input is inactive are probably not representing anything related to that input.
        
        0.61t: den_cycle()

        0.72t: pyr_cycle()

        
    
    Columnar Output:
        1.00t: 

    Cycle Repeats:

    Sporatically:
        ~ 500.00t: Regrowth




## Synapse Pruning and Regrowth

The "edge" overactivity issue (see written notes 04-26):
    Synapses near the edge are sometimes being provided excessive stimulus from nearby axonal areas. This stimulus can lead to synapses being trained disproportionately strengthen in response to this. Possibilities to address this include:
        - Rethinking axon space layout
        - Cleaning up input to axon space (make sure it is as sparse as it should be)
        - Providing mechanisms on the distal synaptic side to ensure sporatic intense stimulation isn't given as much weight.
            - desensitivation / exhaustion
            - other less "organic" methods (last resort)
                - dendritic global statistical inhibition

Rethinking axon space with a bit of cleaning up input seems to be the best approach at this point. Implementing a separate "shared" or "horizontal" space, the width of a synaptic span, would allow inputs to be shared among all cells without regard to spatial concerns. It's not entirely clear how this would or could affect edge problems.

Somewhat related is the recent consideration of a "flags" vector to store extra information about synapses. Hopefully this will end up being unnecessary for tackling learning but may rear its head again. 

Mostly unrelated: 
    - the 2nd (or 3rd depending on how you look at it) spatial vector has been added to the "Synapses" struct and will be integrated soon enough. Inhibition is probably the only major aspect affected by the change from a one dimensional to a two dimensional perspective.


## Inhibition Notes

Currently working fine with:
    - span=8
    - floor=47ish
    - int inhib_power = (ASPINY_SPAN + 1) - (cur_comp_dist);
    - } else if (col_state == cur_comp_state) {
            if ((asp_idx & 0x07 == 4)) {
                win_count += inhib_power;

Biggest weaknesses:
    - highly homogeneous input
    - evenly scaling input, where peaks occur in the signal in very few places (mountains)


Few possible improvements:
    - Run several iterations cascading from most active to least
    - Do a meta-comparison by calculating number of wins, then calculating number of wins of wins
    - Just say fuck it and do it per section (hypercolumn style)


## Cortex Reorganization

- Move cells down to cortical regions
- Move columns up to cortical regions
- Move aspinys down to columns
- Rename columns to spiny stellate
- Hierarchy of physical data structures (envoys):
    - Cortex
        - Regions
            - Cells
                - Stellate
                    - Aspiny
                        - states
                        - ids
                    - Spiny
                        - states
                        - Synapses
                - Pyramidal
                    - states
                    - Dendrites
                        - states
                        - Synapses



## Distal Dendrites 2.0

- Ideally uses same algorithm as proximal for initalizing and cycling synapses.
- Shoot for 32 synapses per (15 - 25 of which are strong enough to signal at any given time).
- Kernel should be changed to:
    - process multiple dendrites at a time 
    - and/or scale workgroup size from 256 to 128 or 64
    - probably both


## Proximal Dendrites 2.0

Option 1: Represent Column as a Dendrite
    - Refactor/rename "column" to "column dendrite" or some such.
    - Allow each cell to specify the layer in which their dendrite resides.
    - Potentially simpler but less flexible

Option 2: Represent Column as a Cell
    - Continue treating column as a cell with only one proximal synapse
    - Other cells treat input as a normal proximal synaptic input
    - More biologically accurate (not that it particularly matters)
    - Might be easier to add other proximal sources.


## Debug Ideas

Look at the center of activity when given a sensory vector with only the very center portion active.


## Learning and Inference 2.0

We have arrived at a stable way to represent input data. Let's implement a more structured and consistent inference mechanism.
    - Dendrites continue to represent the sum of it's dendritic activity.
        - Less linearity needed for distal and apical.
    - Dendrites lean more towards ORing rather than SUMing.
    - Distal dendrites can not cause a cell to fire, only to bring it to the point of being able to fire.
    - Proximal dendrites can only cause a cell to fire if it is fully predictive.
    - Columnar mechanisms (yet undefined) determine when a column should output even though it has no feed-forward (proximal) input. This would represent a predictive output. The details are still unclear.

Refining distal dendrites:
    - scale up to 100+ distal dendrites per cell (3 - 10k per cell)
    - simple threshold computation
    - Less linear, more logarithmic or maybe polylogarithmic (possibly (log s) or (log s)^c)
    - Most distal synapses do nothing and are only plausably "connected".
    - Can uniquely identify > 2^8 patterns
    - Predictions do not inhibit

Bringing in apical dendrites:
    - Concerned with feedback.
    - Non-linear just like distal.

Proximal:
    - Needs to happen first
        - considers activation from proximal synapses from the previous cycle
    - Increase synapse count by a factor of about 2^4.
    - Increase identification capacity to > 2^5 patterns.
    - feed-forward sensory and "precipitous" layers (ex. 2-3 prox connects to 4, 4 prox conn's to thal-cort)

Supplimental:
    - Find a fast, random way to vary the rate of synapse strength gain/loss based on it's distance to max
        - something like:  str += ((127 - str) > (rand >> inv_learning_rate))
        - cheap random: tmp=(xˆ(x<<15)); x=y; y=z; z=w; return w=(wˆ(w>>21))ˆ(tmpˆ(tmp>>4));


## LEARNING 1.0

- Dendrites might be skipped at first -- thresholds will need to be dealt with later.
- Synapses will evaluate whether or not the cell has fired by evaluating the axon state then increase the weight by one (clamped) if their states were positive.
- 


## BISMIT TYPES 1.0

Neuron: (672b)
- Axon (32b) x 256 =
    - Synapse Address<u8> (1b Synapse + 1b Neuron = 2b) x 

-Dendrite (40b) x 16 =
    -Synapses x 16
        -Weight<i4> 16 x (2b) =

-Area of influence = 4 hypercolumns

Column: 11,488
    -Axon (32b) x 1 = 32b
    -Dendrite (40b) x 16 = 640b
    -Neuron (672b) x 1-256 (16 default) = 10,752b
    -Neuron State + History (16b) x 4 = 64b
    Input from previous level
    Outputs to next level

Hypercolumn: 735,232
    -Column (11,488) x 64
    -ActiveOutput (1b) x 1


## Fundamental Building Blocks 1.0

Two things must be communicated for now, the state of the cells on the same level of a (mini)column, and states of columns themselves.


Intra-columnar cells will be represented with vectors of 16 bit unsigned integers (u16). Initially, at least, these will be (16 x 16bit) 256bit.
    [ 0 1 0 1 0 0 1 0 1 0 1 0 1 0 0 1 | 1 0 0 1 0 1 0 1 1 0 1 0 1 0 0 1 | ... | 1 0 0 1 0 1 0 1 0 1 0 0 1 0 1 1 ]

For inter-columnar cells, intensity/size/confidence (representing firing rate and/or any other information sometimes encoded as firing timing offsets {it's all mashed together for now}) must be represented as well:
    An 8 bit (0 - 255) intensity:
        | 0 1 0 1 0 1 0 0 |
    With a 16 bit address space:
        | 1 0 1 0 1 1 0 1 0 1 0 0 1 0 1 1 |
    Giving us a 24 bit message from each column:
        [] 0 1 0 1 0 1 0 0 | 1 0 1 0 1 1 0 1 0 1 0 0 1 0 1 1 ]
    Leaving us 8 bits of space for future use.

Every message leaving a cell will be 8 bits. Every message leaving a column be 8 + 16 bits.
    Obviously columnar outputs will 



## MISC NOTES AND DATA

CELLS:

two primary types of neurons, excitatory pyramidal neurons (~80% of neocortical neurons) and inhibitory interneurons (~20%).


COLUMNS: 


cortex (both hemispheres) is 1.27×10^11 µm2

 diameter of a minicolumn is about 28–40 µm -- 40–50 µm in transverse diameter -- 35–60 µm -- 50 µm with 80 µm spacing -- 30 µm with 50 µm -- 28 µm - 36 µm --

 Cells in 50 µm minicolumn all have the same receptive field; adjacent minicolumns may have very different fields 

neurons that are horizontally more than 0.5 mm (500 µm) from each other do not have overlapping sensory receptive fields - 200–800 µm

 Downwards projecting axons in minicolumns are ≈10 µm in diameter


 neurons in cortex or in neocortex are on the order of 2x10^10

 estimate of 2x10^8 minicolumns -- estimate of 2x10^7-2x10^8 minicolumns

  There are about 100,000,000 cortical minicolumns in the neo-cortex with up to 110 neurons each

 there are 50 to 100 cortical minicolumns in a hypercolumn, each comprising around 80 neurons -- There are about 2×10^8 minicolumns in humans -- Estimates of number of neurons in a minicolumn range from 80-100 neurons -- 11 to 142 neurons per minicolumn --

the neocortex consists of about a half-million of these [cortical] columns

 LAYERS:

  Layer I -- Molecular (plexiform) layer - Few neuronal somata
- Apical dendrites of cells ovated in depper cortical layers
- Axons passing through or making connections in this layer
- Axons arising from within this layer often travel parallel to the layer (hence parallel to the pia)

Layer II -- External granular layer - Small granule cells (local interneurons that inhibit other cortical cells) and some slightly larger pyramidal cells.
- Communications with ipsilateral (same side) cortical areas (via association fibers)

Layer III -- External pyramidal layer - Primarily small to medium sized pyramidal neurons projecting from the cortex.
- Communications with homotopic contralateral cortices (via commissural fibers)

Layer IV -- Internal granular layer - No pyramidal neurons
- Interneurons receiving ascending sensory (thalamocortical) input and projecting to layers II/III.
- Contains dense band of tangential axons that form the outer band of Baillarger, which is hypertrophic and visible to the naked eye in a discrete occipital region (forming the stria Gennari of the striate cortex)

Layer V -- Internal pyramidal (ganglionic) layer - Medium to very large pyramidal neurons
- Major source of cortical output to the brainstem and spinal cord (pyramidal cells from V can make connections with caudal spinal motor neurons).
- Contains a dense band of tangential axons (i.e. the inner band of Baillarger).

Layer VI -- Multiform layer - An assortment of cell types, including a few pyramidal cells.
- Small cells receive input from the thalamus and from cortical layers II, III, and V.
- Axons of the cells in this layer project to superficial cortical layers and also subcortically (e.g. the thalamus)


(1.) Molecular Layer - (input from mostly foreign layer 3 cells)

(2.) (Spatial) External Granular -- Small Pyramidal Granule Cells (inhibitory) -- Few Larger Pyramidal -- Near Top -- high horiz density

(3.) (Sequential) External pyramidal -- small to medium pyramidal -- inter-column output.
    
(4.) (Temporal)  Internal Granular -- No Pyramidal -- Spiny Stellate Cells -- Interneurons -- Communicate with nearby columns horizontally --  dense band of tangential axons

(5.) (Motor Output) Internal Pyramidal (ganglionic) -- Medium to Large Pyramidal -- Output to Brainstem and Spinal Cord -- dense band of tangential

(6.) (Attention) Multiform -- Assorted cell types -- Small cells receive input from thalamus and 2, 3, + 5 -- Outputs to superficial cortical layers and subcortically (thalamus, etc.)

The structure of the neocortex is relatively uniform...
 consisting of six horizontal layers segregated principally by cell type and neuronal connections

 the motor cortex lacks layer IV

 pyramidal neurons in the upper layers II and III project their axons to other areas of neocortex

 while those in the deeper layers V and VI project primarily out of the cortex, e.g. to the thalamus, brainstem, and spinal cord

 Neurons in layer IV receive all of the synaptic connections from outside the cortex (mostly from thalamus) and themselves make short-range, local connections to other cortical layers

  layer IV receives all incoming sensory information and distributes it to the other layers for further processing.

  The supragranular layers consist of layers I to III. The supragranular layers are the primary origin and termination of intracortical connections, which are either associational (i.e., with other areas of the same hemisphere), or commissural (i.e., connections to the opposite hemisphere, primarily through the corpus callosum).

  The internal granular layer, layer IV, receives thalamocortical connections, especially from the specific thalamic nuclei. This is most prominent in the primary sensory cortices

  The infragranular layers, layers V and VI, primarily connect the cerebral cortex with subcortical regions. These layers are most developed in motor cortical areas. The motor areas have extremely small or non-existent granular layers and are often called "agranular cortex". Layer V gives rise to all of the principal cortical efferent projections to basal ganglia, brain stem and spinal cord. Layer VI, the multiform or fusiform layer, projects primarily to the thalamus.


## Random Theories

- Sparsity in cortical networks could be a spatial equivalent to sequences (frames?) in time.
- 


- The cortex is in the business of writing prescriptions:
    - diagnosis information comes in
    - compared against desired outcome
    - prescriptions go out
    - if the correct prescription is made, it will be more likely to be made again

    - the only peice missing is the desired outcome generation
        - once we know that, it should be easy to put the pieces together
        - desired outcome must be an SDR which is etched into memory (hippocampus?) whenever emotional
          positive feedback is given...
            - that SDR must be then able to trigger subsequent SDRs to reconstruct sequence leading
              to positive feedback
        - undesired outcomes must be similarly saved
        - all along the sequence of any event, emotional responses must be steering the prescription in order
          to acheive/avoid outcome (by excitation/inhibition?)

        - memory must be storing information in a two-way traversable structure ... either that or
          the 'motor' layers of the cortex are in charge of that role
