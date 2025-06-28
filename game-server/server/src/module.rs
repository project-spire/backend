use async_trait::async_trait;
use std::error::Error;
use tokio::sync::broadcast;

#[async_trait]
pub trait Module: Send {
    async fn run(&mut self, terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>>;
}

pub struct MainModule {
    init_modules: Vec<Box<dyn Module>>,
    start_modules: Vec<Box<dyn Module>>,
    stop_modules: Vec<Box<dyn Module>>,
    update_modules: Vec<Box<dyn Module>>,
}

impl MainModule {
    async fn init(&mut self, mut terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
        for mut module in &mut self.init_modules.drain(..) {
            tokio::select! {
                result = module.run(terminate_rx.resubscribe()) => match result {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                },
                _ = terminate_rx.recv() => break,
            }
        }

        Ok(())
    }

    async fn start(&mut self, mut terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
        for module in &mut self.start_modules {
            tokio::select! {
                result = module.run(terminate_rx.resubscribe()) => match result {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                },
                _ = terminate_rx.recv() => break,
            }
        }

        Ok(())
    }

    async fn stop(&mut self, mut terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
        for module in &mut self.stop_modules {
            tokio::select! {
                result = module.run(terminate_rx.resubscribe()) => match result {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                },
                _ = terminate_rx.recv() => break,
            }
        }

        Ok(())
    }

    async fn update(&mut self, mut terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
        for module in &mut self.update_modules {
            tokio::select! {
                result = module.run(terminate_rx.resubscribe()) => match result {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                },
                _ = terminate_rx.recv() => break,
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for MainModule {
    async fn run(&mut self, mut terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
        self.init(terminate_rx.resubscribe()).await?;
        self.start(terminate_rx.resubscribe()).await?;

        if self.update_modules.is_empty() {
            return Ok(());
        }

        loop {
            tokio::select! {
                result = self.update(terminate_rx.resubscribe()) => match result {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                },
                _ = terminate_rx.recv() => return Ok(()),
            }
        }
    }
}
