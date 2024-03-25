use void_core::{ICmdReceiver, Result,ISubject, ISystem};

use crate::{IoCmd, IoEngine, IoEvent};

impl<S, R> ISystem for IoEngine<S, R>
where
    S: ISubject<E = IoEvent>,
    R: ICmdReceiver<IoCmd>,
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
