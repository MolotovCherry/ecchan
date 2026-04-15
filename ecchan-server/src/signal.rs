use std::{io, sync::Arc};

use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    sync::SetOnce,
    task,
};

pub fn shutdown_handler() -> io::Result<ShutdownSignal> {
    let sh_signal = ShutdownSignal::new();

    let mut sighup = signal(SignalKind::hangup())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    task::spawn({
        let signal = sh_signal.clone();

        async move {
            select! {
                biased;

                _ = sigint.recv() => (),
                _ = sigterm.recv() => (),
                _ = sighup.recv() => (),
            }

            signal.notify();
        }
    });

    Ok(sh_signal)
}

#[derive(Clone)]
pub struct ShutdownSignal(Arc<SetOnce<()>>);

impl ShutdownSignal {
    pub fn new() -> Self {
        Self(Arc::new(SetOnce::new()))
    }

    /// Set the state to signalled
    pub fn notify(&self) {
        _ = self.0.set(());
    }

    pub async fn wait(&self) {
        self.0.wait().await;
    }
}
