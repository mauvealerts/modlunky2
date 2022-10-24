use std::time::Duration;

use clap::{Parser, Subcommand};
use rand::distributions::Uniform;
use tokio::{select, sync::broadcast};
use tokio_graceful_shutdown::{IntoSubsystem as _, SubsystemHandle, Toplevel};

use ml2_mods::{
    data::Change,
    local::{
        cache::{ModCache, ModCacheHandle},
        disk::DiskMods,
    },
    manager::{ModManager, ModManagerHandle, ModSource, DEFAULT_RECEIVING_INTERVAL},
    spelunkyfyi::{
        http::{HttpApiMods, DEFAULT_SERVICE_ROOT},
        web_socket::{
            WebSocketClient, DEFAULT_MAX_PING_INTERVAL, DEFAULT_MIN_PING_INTERVAL,
            DEFAULT_PONG_TIMEOUT,
        },
    },
};
use ml2_net::http::HttpClient;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'i', long)]
    install_path: String,
    #[clap(short = 't', long)]
    token: Option<String>,
    #[clap(long, default_value_t = DEFAULT_SERVICE_ROOT.to_string())]
    service_root: String,

    // We poll and scan more aggressively to make the demo snappier
    #[clap(long, default_value_t = 15)]
    api_poll_interval_sec: u64,
    #[clap(long, default_value_t = 1)]
    api_poll_delay_sec: u64,
    #[clap(long, default_value_t = 5)]
    local_scan_interval_sec: u64,
    #[clap(long, default_value_t = DEFAULT_RECEIVING_INTERVAL.as_millis() as u64)]
    receiving_interval_millis: u64,

    #[clap(long)]
    ping_min_interval: Option<u64>,
    #[clap(long)]
    ping_max_interval: Option<u64>,
    #[clap(long)]
    pong_timeout: Option<u64>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get { id: String },
    List {},
    Remove { id: String },
    InstallLocal { source: String, id: String },
    InstallRemote { code: String },
    UpdateLocal { source: String, id: String },
    UpdateRemote { code: String },
    Run {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let api_client = cli
        .token
        .as_ref()
        .map(|token| HttpApiMods::new(&cli.service_root, token, HttpClient::new()))
        .transpose()?;

    let (detected_tx, detected_rx) = broadcast::channel(10);
    let (mod_cache, mod_cache_handle) = ModCache::new(
        api_client.clone(),
        Duration::from_secs(cli.api_poll_interval_sec),
        Duration::from_secs(cli.api_poll_delay_sec),
        detected_tx,
        DiskMods::new(&cli.install_path),
        Duration::from_secs(cli.local_scan_interval_sec),
    );

    let (changes_tx, changes_rx) = broadcast::channel(10);
    let (manager, manager_handle) = ModManager::new(
        api_client.clone(),
        mod_cache.clone(),
        changes_tx,
        detected_rx,
        Duration::from_millis(cli.receiving_interval_millis),
    );

    let ping_interval_dist = Uniform::new(
        cli.ping_min_interval
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_MIN_PING_INTERVAL),
        cli.ping_max_interval
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_MAX_PING_INTERVAL),
    );
    let pong_timeout = cli
        .pong_timeout
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_PONG_TIMEOUT);
    let web_socket_client = cli
        .token
        .as_ref()
        .map(|token| {
            WebSocketClient::new(
                &cli.service_root,
                token,
                manager_handle.clone(),
                ping_interval_dist,
                pong_timeout,
            )
        })
        .transpose()?;

    let mut toplevel = Toplevel::new()
        .catch_signals()
        .start("ModCache", mod_cache.into_subsystem())
        .start("ModManager", manager.into_subsystem());
    if let Some(web_socket_client) = web_socket_client {
        toplevel = toplevel.start("WebSocket", web_socket_client.into_subsystem());
    }
    toplevel
        .start("CLI", |h| {
            run(h, cli.command, manager_handle, mod_cache_handle, changes_rx)
        })
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await?;

    Ok(())
}

async fn run(
    subsystem: SubsystemHandle,
    cmd: Commands,
    manager: ModManagerHandle,
    cache: ModCacheHandle,
    mut mods_rx: broadcast::Receiver<Change>,
) -> anyhow::Result<()> {
    cache.ready().await;
    match cmd {
        Commands::Get { id } => {
            println!("{:#?}", manager.get(&id).await?);
        }
        Commands::List {} => {
            println!("{:#?}", manager.list().await?);
        }
        Commands::Remove { id } => {
            println!("{:#?}", manager.remove(&id).await?);
        }
        Commands::InstallLocal { source, id } => {
            let package = ModSource::Local {
                source_path: source,
                dest_id: id,
            };
            println!("{:#?}", manager.install(&package).await?);
        }
        Commands::InstallRemote { code } => {
            let package = ModSource::Remote { code };
            println!("{:#?}", manager.install(&package).await?);
        }
        Commands::UpdateLocal { source, id } => {
            let package = ModSource::Local {
                source_path: source,
                dest_id: id,
            };
            println!("{:#?}", manager.install(&package).await?);
        }
        Commands::UpdateRemote { code } => {
            let package = ModSource::Remote { code };
            println!("{:#?}", manager.update(&package).await?);
        }
        Commands::Run {} => loop {
            select! {
                _ = subsystem.on_shutdown_requested() => break,
                Ok(change) = mods_rx.recv() => println!("{:#?}", change),
            }
        },
    }
    subsystem.request_global_shutdown();
    Ok(())
}
