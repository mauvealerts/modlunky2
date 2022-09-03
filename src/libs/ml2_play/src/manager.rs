use async_trait::async_trait;
use derivative::Derivative;
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

use crate::{
    binary::Binary,
    config::{LoadOrder, Playlunky},
    data::{Event, LoadOrderConfig, PlaylunkyConfig, Version},
    Error, Result,
};

#[derive(Derivative)]
#[derivative(Debug)]
enum Command {
    Launch {
        version: Version,
        #[derivative(Debug = "ignore")]
        resp: oneshot::Sender<Result<()>>,
    },
    ReadLoadOrder {
        #[derivative(Debug = "ignore")]
        resp: oneshot::Sender<Result<LoadOrderConfig>>,
    },
    ReadPlaylunkyConfig {
        #[derivative(Debug = "ignore")]
        resp: oneshot::Sender<Result<PlaylunkyConfig>>,
    },
    WriteLoadOrder {
        load_order: LoadOrderConfig,

        #[derivative(Debug = "ignore")]
        resp: oneshot::Sender<Result<()>>,
    },
    WritePlaylunkyConfig {
        config: PlaylunkyConfig,

        #[derivative(Debug = "ignore")]
        resp: oneshot::Sender<Result<()>>,
    },
}

pub struct Manager {
    binary: Binary,
    commands_rx: mpsc::Receiver<Command>,
    events_tx: mpsc::Sender<Event>,
    load_order: LoadOrder,
    playlunky: Playlunky,
}

pub struct Handle {
    commands_tx: mpsc::Sender<Command>,
}

impl Manager {
    pub fn new(
        events_tx: mpsc::Sender<Event>,
        install_dir: &str,
        ml2_path: &str,
    ) -> (Self, Handle) {
        let (commands_tx, commands_rx) = mpsc::channel(1);
        let manager = Self {
            binary: Binary::new(install_dir, ml2_path),
            commands_rx,
            events_tx,
            load_order: LoadOrder::new(install_dir),
            playlunky: Playlunky::new(install_dir),
        };
        let handle = Handle { commands_tx };
        (manager, handle)
    }

    async fn handle_command(&self, cmd: Command) {
        match cmd {
            Command::Launch { version, resp } => {
                let tx = self.events_tx.clone();
                let _ = resp.send(self.binary.launch(version, tx).await);
            }
            Command::ReadLoadOrder { resp } => {
                let _ = resp.send(self.load_order.read().await);
            }
            Command::ReadPlaylunkyConfig { resp } => {
                let _ = resp.send(self.playlunky.read().await);
            }
            Command::WriteLoadOrder { load_order, resp } => {
                let _ = resp.send(self.load_order.write(load_order).await);
            }
            Command::WritePlaylunkyConfig { config, resp } => {
                let _ = resp.send(self.playlunky.write(config).await);
            }
        }
    }
}

#[async_trait]
impl IntoSubsystem<Error> for Manager {
    async fn run(mut self, subsys: SubsystemHandle) -> Result<()> {
        loop {
            select! {
                _ = subsys.on_shutdown_requested() => break,
                Some(cmd) = self.commands_rx.recv() => self.handle_command(cmd).await,
            }
        }
        Ok(())
    }
}

impl Handle {
    pub async fn launch(&self, version: Version) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.commands_tx
            .send(Command::Launch { version, resp: tx })
            .await?;
        rx.await?
    }

    pub async fn read_load_order(&self) -> Result<LoadOrderConfig> {
        let (tx, rx) = oneshot::channel();
        self.commands_tx
            .send(Command::ReadLoadOrder { resp: tx })
            .await?;
        rx.await?
    }

    pub async fn read_playlunky_config(&self) -> Result<PlaylunkyConfig> {
        let (tx, rx) = oneshot::channel();
        self.commands_tx
            .send(Command::ReadPlaylunkyConfig { resp: tx })
            .await?;
        rx.await?
    }

    pub async fn write_load_order(&self, load_order: LoadOrderConfig) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.commands_tx
            .send(Command::WriteLoadOrder {
                load_order,
                resp: tx,
            })
            .await?;
        rx.await?
    }

    pub async fn write_playlunky_config(&self, config: PlaylunkyConfig) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.commands_tx
            .send(Command::WritePlaylunkyConfig { config, resp: tx })
            .await?;
        rx.await?
    }
}
