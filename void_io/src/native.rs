use void_core::{CmdReceiver, Result, Subject, System};

use crate::{IoCmd, IoEngine, IoEvent};

impl<S, R> System for IoEngine<S, R>
where
    S: Subject<E = IoEvent>,
    R: CmdReceiver<IoCmd>,
{
    type C = IoCmd;

    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(cmd) = self.receiver.recv().await {
                self.handle_cmd(cmd)?;
            }
        }
    }

    fn run_blocking(&mut self) -> Result<()> {
        let cmd = self.receiver.recv_blockding().unwrap();
        self.handle_cmd(cmd)?;
        Ok(())
    }
}
