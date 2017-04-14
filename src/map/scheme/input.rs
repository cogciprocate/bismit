
#[derive(PartialEq, Debug, Clone)]
pub enum EncoderScheme {
    None,
    Custom { layer_count: usize },
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Zeros,
    IdxStreamer { file_name: String, cyc_per: usize, scale: f32, loop_frames: u32 },
    // IdxStreamerLoop { file_name: String, cyc_per: usize, scale: f32, loop_frames: u32 },
    GlyphSequences { seq_lens: (usize, usize), seq_count: usize, scale: f32, hrz_dims: (u32, u32) },
    SensoryTract,

    // Possibly remove me eventually:
    ScalarSequence { range: (f32, f32), incr: f32 },
    ReversoScalarSequence { range: (f32, f32), incr: f32 },
    VectorEncoder { ranges: Vec<(f32, f32)> },
    ScalarSdrGradiant { range: (f32, f32), way_span: f32, incr: f32 },
}

impl EncoderScheme {
    pub fn is_some(&self) -> bool {
        match *self {
            EncoderScheme::None { .. } => false,
            _ => true,
        }
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn layer_count(&self) -> usize {
        match *self {
            EncoderScheme::None => 0,
            EncoderScheme::GlyphSequences { .. } => 2,
            EncoderScheme::ReversoScalarSequence { .. } => 2,
            EncoderScheme::VectorEncoder { ref ranges } => ranges.len(),
            EncoderScheme::Custom { layer_count } => layer_count,
            // EncoderScheme::SensoryTract { ref dims, .. } => dims.len(),
            _ => 1,
        }
    }
}
