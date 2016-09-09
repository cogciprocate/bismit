mod thalamus;
mod external_pathway;

pub use self::external_pathway::{ExternalPathway, ExternalPathwayFrame, ExternalPathwayTract,
    ExternalPathwayEncoder, ExternalPathwayLayer};
pub use self::thalamus::{Thalamus};
pub use cmn::TractFrameMut;
pub use map::LayerTags;