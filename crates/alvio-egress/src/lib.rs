pub mod error;
pub mod manager;
pub mod session;
pub mod supervisor;
pub mod tap;

pub use error::{EgressError, EgressResult};
pub use manager::RecordingManager;
pub use session::{RecordingConfig, RecordingOutput, RecordingStatus};
pub use supervisor::{FfmpegCommandBuilder, FfmpegSupervisor, OutputFormat};
pub use tap::MediaTap;
