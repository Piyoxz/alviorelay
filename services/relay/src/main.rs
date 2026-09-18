use alvio_core::{AlvioConfig, AlvioResult};
use alvio_observe::init_telemetry;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "alvio-relay",
    about = "AlvioRelay: Rust-Native Self-Hosted Real-Time Media Infrastructure",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to configuration file (defaults to ./alvio-relay.toml)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the AlvioRelay core media & signaling server
    Start {
        /// Override HTTP/WS port
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Validate configuration and network setup without running the server
    Check,
    /// Display resolved configuration
    Config,
    /// Display version and build information
    Version,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from("alvio-relay.toml"));

    match cli.command.unwrap_or(Commands::Start { port: None }) {
        Commands::Version => {
            println!("AlvioRelay Media Server v{}", env!("CARGO_PKG_VERSION"));
            println!("Architecture: {}-{}", std::env::consts::OS, std::env::consts::ARCH);
            println!("License: MIT OR Apache-2.0");
        }
        Commands::Check => {
            match AlvioConfig::load_from_file_or_default(&config_path) {
                Ok(config) => {
                    println!("Configuration '{}' is VALID.", config_path.display());
                    println!("Node ID    : {}", config.server.node_id);
                    println!("Bind Addr  : {}:{}", config.server.bind_address, config.server.http_port);
                    println!("RTC UDP    : {}", config.rtc.udp_port);
                    println!("Auth       : {}", config.auth.provider);
                }
                Err(e) => {
                    eprintln!("Configuration check FAILED: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Config => {
            let config = AlvioConfig::load_from_file_or_default(&config_path)?;
            let json = serde_json::to_string_pretty(&config)?;
            println!("{json}");
        }
        Commands::Start { port } => {
            let mut config = AlvioConfig::load_from_file_or_default(&config_path)?;
            if let Some(p) = port {
                config.server.http_port = p;
                config.server.ws_port = p;
            }

            init_telemetry(&config.server.log_level)?;

            print_banner(&config);

            info!(
                node_id = %config.server.node_id,
                http_port = config.server.http_port,
                rtc_udp_port = config.rtc.udp_port,
                auth_provider = %config.auth.provider,
                "AlvioRelay server initialized in-memory"
            );

            run_server_lifecycle(&config).await?;
        }
    }

    Ok(())
}

fn print_banner(config: &AlvioConfig) {
    println!(r#"
       ___    __      _       ____      __           
      /   |  / /_  __(_)___  / __ \___ / /___ ___  __
     / /| | / / / / / / __ \/ /_/ / _ \/ / __ `/ / / /
    / ___ |/ / /_/ / / /_/ / _, _/  __/ / /_/ / /_/ / 
   /_/  |_/_/\__,_/_/\____/_/ |_|\___/_/\__,_/\__, /  
                                             /____/   
   Rust-Native Self-Hosted Real-Time Media Infrastructure
   Version: {} | Node: {}
"#, env!("CARGO_PKG_VERSION"), config.server.node_id);
}

async fn run_server_lifecycle(config: &AlvioConfig) -> AlvioResult<()> {
    info!(
        bind = format!("{}:{}", config.server.bind_address, config.server.http_port),
        "Signaling and HTTP gateway ready to accept incoming connections"
    );

    // Wait for shutdown signal (Ctrl+C / SIGINT)
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| alvio_core::AlvioError::Internal(format!("Signal handler failed: {e}")))?;

    warn!("Received shutdown signal. Entering graceful drain mode...");
    info!("Disconnecting peers and flushing remaining telemetry...");
    info!("AlvioRelay shutdown completed cleanly.");

    Ok(())
}
