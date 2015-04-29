=== Fundamental Building Blocks ===
	Two things must be communicated for now, the state of the cells on the same level of a (mini)column, and states of columns themselves.

	
	Integerra-columnar cells will be represented with vectors of 16 bit unsigned integers (u16). Initially, at least, these will be (16 x 16bit) 256bit.
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

Layer IV -- Integerernal granular layer - No pyramidal neurons
- Integererneurons receiving ascending sensory (thalamocortical) input and projecting to layers II/III.
- Contains dense band of tangential axons that form the outer band of Baillarger, which is hypertrophic and visible to the naked eye in a discrete occipital region (forming the stria Gennari of the striate cortex)

Layer V -- Integerernal pyramidal (ganglionic) layer - Medium to very large pyramidal neurons
- Major source of cortical output to the brainstem and spinal cord (pyramidal cells from V can make connections with caudal spinal motor neurons).
- Contains a dense band of tangential axons (i.e. the inner band of Baillarger).

Layer VI -- Multiform layer - An assortment of cell types, including a few pyramidal cells.
- Small cells receive input from the thalamus and from cortical layers II, III, and V.
- Axons of the cells in this layer project to superficial cortical layers and also subcortically (e.g. the thalamus)


(1.) Molecular Layer - (input from mostly foreign layer 3 cells)

(2.) (Spatial) External Granular -- Small Pyramidal Granule Cells (inhibitory) -- Few Larger Pyramidal -- Near Top -- high horiz density

(3.) (Sequential) External pyramidal -- small to medium pyramidal -- inter-column output.
	
(4.) (Temporal)  Integerernal Granular -- No Pyramidal -- Spiny Stellate Cells -- Integererneurons -- Communicate with nearby columns horizontally --  dense band of tangential axons

(5.) (Motor Output) Integerernal Pyramidal (ganglionic) -- Medium to Large Pyramidal -- Output to Brainstem and Spinal Cord -- dense band of tangential

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






BISMIT TYPES:


Neuron: (672b)
	-Axon (32b) x 256 =
		-Synapse Address<u8> (1b Synapse + 1b Neuron = 2b) x 

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







(Integerel  Core(TM) i5-2500K: Local Memory Size = 32768)
