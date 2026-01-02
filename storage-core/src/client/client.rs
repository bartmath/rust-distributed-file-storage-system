use crate::commands::CliCommand;

pub(super) struct Client {}

impl Client {
    pub(super) fn new() -> Self {
        Client {}
    }
    pub(super) async fn handle_command(&self, _cmd: CliCommand) -> anyhow::Result<()> {
        Ok(())
    }
}
