use crate::error::{EgressError, EgressResult};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

/// Target container format for media egress recordings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Mp4,
    WebM,
    Hls,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::WebM => "webm",
            Self::Hls => "m3u8",
        }
    }
}

/// Command builder for constructing isolated FFmpeg muxing commands.
pub struct FfmpegCommandBuilder;

impl FfmpegCommandBuilder {
    pub fn build_args(format: OutputFormat, output_path: &str) -> Vec<String> {
        let mut args = vec![
            "-y".to_string(),               // Overwrite output
            "-loglevel".to_string(),        // Controlled log output
            "warning".to_string(),
            "-i".to_string(),               // Read from standard input pipe
            "pipe:0".to_string(),
        ];

        match format {
            OutputFormat::Mp4 => {
                args.extend([
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "aac".to_string(),
                    "-movflags".to_string(), "+faststart".to_string(),
                    output_path.to_string(),
                ]);
            }
            OutputFormat::WebM => {
                args.extend([
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "libopus".to_string(),
                    output_path.to_string(),
                ]);
            }
            OutputFormat::Hls => {
                args.extend([
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "aac".to_string(),
                    "-f".to_string(), "hls".to_string(),
                    "-hls_time".to_string(), "4".to_string(),
                    "-hls_list_size".to_string(), "0".to_string(),
                    output_path.to_string(),
                ]);
            }
        }

        args
    }
}

/// Process supervisor managing the lifecycle of an isolated FFmpeg recording subprocess.
pub struct FfmpegSupervisor {
    child: Option<Child>,
    format: OutputFormat,
    output_path: String,
    is_mock: bool,
}

impl FfmpegSupervisor {
    /// Spawns an isolated FFmpeg process.
    ///
    /// If FFmpeg is not installed on the system, gracefully falls back to mock mode
    /// to avoid crashing development or lightweight server environments.
    pub fn spawn(format: OutputFormat, output_path: &str) -> EgressResult<Self> {
        let args = FfmpegCommandBuilder::build_args(format, output_path);

        let spawn_result = Command::new("ffmpeg")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        match spawn_result {
            Ok(child) => {
                info!(output = %output_path, ?format, "Spawned isolated FFmpeg worker process");
                Ok(Self {
                    child: Some(child),
                    format,
                    output_path: output_path.to_string(),
                    is_mock: false,
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("FFmpeg binary not found on PATH. Operating in simulated egress mode.");
                Ok(Self {
                    child: None,
                    format,
                    output_path: output_path.to_string(),
                    is_mock: true,
                })
            }
            Err(e) => {
                error!(error = %e, "Failed to spawn FFmpeg supervisor");
                Err(EgressError::Ffmpeg(format!("Failed to spawn FFmpeg: {e}")))
            }
        }
    }

    /// Feeds a chunk of media data into the FFmpeg stdin pipe.
    pub async fn write_data(&mut self, data: &[u8]) -> EgressResult<()> {
        if self.is_mock {
            return Ok(());
        }

        if let Some(ref mut child) = self.child {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(data).await?;
            }
        }
        Ok(())
    }

    /// Closes the input pipe and awaits orderly termination of the FFmpeg subprocess.
    pub async fn stop(&mut self) -> EgressResult<()> {
        if self.is_mock {
            return Ok(());
        }

        if let Some(mut child) = self.child.take() {
            // Close stdin to signal EOF to FFmpeg
            drop(child.stdin.take());

            match child.wait().await {
                Ok(status) => {
                    debug!(?status, "FFmpeg worker process exited successfully");
                    Ok(())
                }
                Err(e) => {
                    error!(error = %e, "FFmpeg process error during termination");
                    Err(EgressError::Ffmpeg(format!("FFmpeg wait error: {e}")))
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn is_mock(&self) -> bool {
        self.is_mock
    }

    pub fn output_path(&self) -> &str {
        &self.output_path
    }

    pub fn format(&self) -> OutputFormat {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_command_builder_mp4() {
        let args = FfmpegCommandBuilder::build_args(OutputFormat::Mp4, "/tmp/out.mp4");
        assert!(args.contains(&"-movflags".to_string()));
        assert!(args.contains(&"+faststart".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/out.mp4");
    }

    #[test]
    fn test_ffmpeg_command_builder_webm() {
        let args = FfmpegCommandBuilder::build_args(OutputFormat::WebM, "/tmp/out.webm");
        assert!(args.contains(&"libopus".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/out.webm");
    }

    #[test]
    fn test_ffmpeg_command_builder_hls() {
        let args = FfmpegCommandBuilder::build_args(OutputFormat::Hls, "/tmp/live.m3u8");
        assert!(args.contains(&"-hls_time".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/live.m3u8");
    }
}
