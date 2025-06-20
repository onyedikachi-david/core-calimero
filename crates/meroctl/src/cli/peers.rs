use calimero_server_primitives::admin::GetPeersCountResponse;
use clap::Parser;
use comfy_table::{Cell, Color, Table};
use const_format::concatcp;
use eyre::{OptionExt, Result as EyreResult};
use reqwest::Client;

use crate::cli::Environment;
use crate::common::{do_request, RequestType};
use crate::output::Report;

pub const EXAMPLES: &str = r"
  #
  $ meroctl --node node1 peers
";

#[derive(Debug, Parser)]
#[command(about = "Return the number of connected peers")]
#[command(after_help = concatcp!(
    "Examples:",
    EXAMPLES
))]
pub struct PeersCommand;

impl Report for GetPeersCountResponse {
    fn report(&self) {
        let mut table = Table::new();
        let _ = table.set_header(vec![Cell::new("Connected Peers").fg(Color::Blue)]);
        let _ = table.add_row(vec![self.count.to_string()]);
        println!("{table}");
    }
}

impl PeersCommand {
    pub async fn run(&self, environment: &Environment) -> EyreResult<()> {
        let connection = environment
            .connection
            .as_ref()
            .ok_or_eyre("No connection configured")?;

        let mut url = connection.api_url.clone();
        url.set_path("admin-api/dev/peers");

        let response: GetPeersCountResponse = do_request(
            &Client::new(),
            url,
            None::<()>,
            connection.auth_key.as_ref(),
            RequestType::Get,
        )
        .await?;

        environment.output.write(&response);

        Ok(())
    }
}
