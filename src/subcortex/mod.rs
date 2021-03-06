mod subcortex;
mod cerebellum;
mod thalamus;
mod tract_channel;
mod input_generator;
mod test_nucleus;
// mod cortical_sampler;

pub use self::subcortex::{Subcortex, SubcorticalNucleus, SubcorticalNucleusLayer,  };
pub use self::thalamus::{Thalamus, /*InputGeneratorTract, InputGenerator, InputGeneratorFrame,
    InputGeneratorEncoder, InputGeneratorLayer*/};
pub use self::cerebellum::Cerebellum;

pub use self::input_generator::{InputGenerator, InputGeneratorFrame,
    InputGeneratorTract, InputGeneratorEncoder, InputGeneratorLayer};

pub use self::tract_channel::{tract_channel_single_u8, tract_channel_single_i8,
    TractBuffer, TractSender, TractReceiver, FutureSend, FutureRecv,
    tract_channel_single_u8_send_only, tract_channel_single_u8_recv_only,
    WriteBuffer, ReadBuffer, /*FutureWriteGuardUntyped,*/ FutureReadGuardVec,
    /*WriteGuardUntyped,*/ ReadGuardVec};

pub use self::test_nucleus::TestScNucleus;