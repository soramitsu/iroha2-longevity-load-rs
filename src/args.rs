use async_trait::async_trait;
use color_eyre::eyre::Result;
use std::io::{BufWriter, Write};

/// trait to encapsulate common attributes of the commands and sub-commands.
#[async_trait]
pub trait RunArgs {
    /// run the given command.
    ///
    /// # errors
    /// if inner command fails.
    async fn run<T: Write + Send>(self, writer: &mut BufWriter<T>) -> Result<()>;
}
