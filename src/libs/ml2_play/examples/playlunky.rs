use anyhow::anyhow;
use clap::{Parser, Subcommand};
use directories::{BaseDirs, ProjectDirs};

use ml2_play::config::{LoadOrder, Playlunky};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'i', long)]
    install_path: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Playlunky,
    LoadOrder,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let subpath = ProjectDirs::from("", "spelunky.fyi", "modlunky2")
        .ok_or_else(|| anyhow!("Unable to determine modlunky2 directory"))?
        .project_path()
        .to_owned();
    let _ml2_dir = BaseDirs::new()
        .ok_or_else(|| anyhow!("Unable to determine local data directory"))?
        .data_local_dir()
        .join(subpath);

    match cli.command {
        Commands::Playlunky => {
            let config = Playlunky::new(&cli.install_path).read().await?;
            println!("{:#?}", config);
        }
        Commands::LoadOrder => {
            let order = LoadOrder::new(&cli.install_path).read().await?;
            println!("{:#?}", order);
        }
    }

    Ok(())
}
