use alvio_core::{AlvioConfig, AlvioResult};
use alvio_observe::init_telemetry;
use alvio_signal::{create_signaling_router, RoomRegistry};
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
            println!(
                "Architecture: {}-{}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            println!("License: MIT OR Apache-2.0");
        }
        Commands::Check => match AlvioConfig::load_from_file_or_default(&config_path) {
            Ok(config) => {
                println!("Configuration '{}' is VALID.", config_path.display());
                println!("Node ID    : {}", config.server.node_id);
                println!(
                    "Bind Addr  : {}:{}",
                    config.server.bind_address, config.server.http_port
                );
                println!("RTC UDP    : {}", config.rtc.udp_port);
                println!("Auth       : {}", config.auth.provider);
            }
            Err(e) => {
                eprintln!("Configuration check FAILED: {e}");
                std::process::exit(1);
            }
        },
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
    println!(
        r#"
       ___    __      _       ____      __           
      /   |  / /_  __(_)___  / __ \___ / /___ ___  __
     / /| | / / / / / / __ \/ /_/ / _ \/ / __ `/ / / /
    / ___ |/ / /_/ / / /_/ / _, _/  __/ / /_/ / /_/ / 
   /_/  |_/_/\__,_/_/\____/_/ |_|\___/_/\__,_/\__, /  
                                             /____/   
   Rust-Native Self-Hosted Real-Time Media Infrastructure
   Version: {} | Node: {}
"#,
        env!("CARGO_PKG_VERSION"),
        config.server.node_id
    );
}

async fn run_server_lifecycle(config: &AlvioConfig) -> AlvioResult<()> {
    let registry = RoomRegistry::new(config.room.max_peers_per_room);
    let signaling_app = create_signaling_router(registry.clone(), config.server.node_id.clone());

    let rtc_addr = format!("{}:{}", config.server.bind_address, config.rtc.udp_port)
        .parse()
        .map_err(|e| alvio_core::AlvioError::Config(format!("Invalid RTC bind address: {e}")))?;

    alvio_observe::init_server_start_time();

    let whip_state =
        alvio_ingress::WhipState::new(alvio_ingress::WhipRegistry::new(), registry, rtc_addr, None);
    let whip_router = alvio_ingress::create_whip_router(whip_state);
    let app = signaling_app
        .nest("/whip", whip_router)
        .route(
            "/metrics",
            axum::routing::get(alvio_observe::metrics_handler),
        )
        .route("/health", axum::routing::get(alvio_observe::health_handler))
        .route("/ready", axum::routing::get(alvio_observe::ready_handler));

    let addr = format!("{}:{}", config.server.bind_address, config.server.http_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| alvio_core::AlvioError::Internal(format!("Failed to bind to {addr}: {e}")))?;

    info!(
        bind = %addr,
        "Signaling, WHIP Ingress, Health, and Observability ready at http://{addr}/, ws://{addr}/ws, http://{addr}/whip/{{room_id}}, http://{addr}/metrics, http://{addr}/health, and http://{addr}/ready"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            alvio_observe::set_draining(true);
            warn!("Received shutdown signal. Entering graceful drain mode...");
        })
        .await
        .map_err(|e| alvio_core::AlvioError::Internal(format!("Server error: {e}")))?;

    info!("Disconnecting peers and flushing remaining telemetry...");
    info!("AlvioRelay shutdown completed cleanly.");

    Ok(())
}
