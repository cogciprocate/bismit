// use input_source::InputTract;

// pub trait ProtoInputTract {
//     fn create() -> Box<InputTract>;
// }

#[derive(PartialEq, Debug, Clone)]
pub enum Protoinput {
    None,
    // Custom(Box<ProtoInputTract>),
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Zeros,
    IdxStreamer { file_name: String, cyc_per: usize, scale: f32, loop_frames: u32 },
    // IdxStreamerLoop { file_name: String, cyc_per: usize, scale: f32, loop_frames: u32 },
    GlyphSequences { seq_lens: (usize, usize), seq_count: usize, scale: f32, hrz_dims: (u32, u32) },
}

impl Protoinput {
    pub fn is_some(&self) -> bool {
        match self {
            &Protoinput::None => false,
            _ => true,
        }
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn layer_count(&self) -> usize {
        match self {
            &Protoinput::GlyphSequences { .. } => 2,
            _ => 1,
        }
    }
}
