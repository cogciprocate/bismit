#BISMIT

Biologically Inspired Sensory Motor Inference Tool: 
A model of the neocortex for interpreting data and taking action based on past experience.

Bismit is one of the first of the next paradigm of learning networks. Going beyond Bayesian and hidden Markov models, incorporating ideas from the theory of hierarchical temporal memory, such as sparse distributed representations and temporal context, Bismit is a model which incorporates our most recent and up-to-date findings about the neocortex. It is not a typical "Machine Learning" platform and does not use traditional statistical methods. 

Intended to be used as a platform for quickly prototyping and testing completely different arrangements of layer structure within a cortical area, as well as different arrangements of connections between cortical areas. Layers can be composed using a simple syntax using intelligent defaults and optional arguments (such as the .apical argument below):

```
proto_layer_maps.add(ProtoLayerMap::new("visual", Sensory)
		.layer("motor_in", 1, layer::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 0, layer::EFFERENT_INPUT, Axonal(Spatial))
		.layer("aff_in", 0, layer::AFFERENT_INPUT, Axonal(Spatial))
		.layer("out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))
		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 600)) 
		.layer("iv_inhib", 0, layer::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 1, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(0, 5, vec!["iii"], 1200).apical(vec!["eff_in"])));

proto_layer_maps.add(ProtoLayerMap::new("external", Thalamic)
		.layer("ganglion", 1, layer::AFFERENT_OUTPUT | layer::AFFERENT_INPUT, Axonal(Spatial)));
```

Likewise for areas:

```
let proto_area_maps = ProtoAreaMaps::new()
	.area_ext("v0", "external", 64, 64,
		Protoinput::IdxReader { 
			file_name: "data/train-images-idx3-ubyte", 
			repeats: REPEATS_PER_IMAGE, 
			scale: 1.3,
		},
		None, 
		Some(vec!["v1"]),
	)

	.area("v1", "visual", 64, 64,
		Some(vec![Protofilter::new("retina", Some("filters.cl"))]),			
		Some(vec!["b1"]),
	)

	.area("b1", "visual", 48, 48, None,	Some(vec!["a1"])),

	.area("a1", "visual", 32, 32, None, None);
```

Bismit is written in Rust and OpenCL C and is in an unstable pre-alpha stage. Full basic functionality is expected by the end 2015.



[//]: # (POSTSCRIPT (move this elsewhere): )

[//]: # (We are now within close reach, eight to ten years or a factor of about a thousand, of matching the processing power of the human neocortex on a $1000 device. By harnessing the inexpensive parallel computing power offered by GPGPUs (or any other OpenCL device) we can build a platform to best utilize that amount of power, intended for maximum performance with maximum flexibility in computing device interoperability.)
