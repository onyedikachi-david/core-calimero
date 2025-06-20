use calimero_context_config::types::Capability as ConfigCapability;
use calimero_primitives::alias::Alias;
use calimero_primitives::context::ContextId;
use calimero_primitives::identity::PublicKey;
use calimero_server_primitives::admin::GrantPermissionResponse;
use clap::Parser;
use eyre::OptionExt;
use reqwest::Client;

use super::Capability;
use crate::cli::Environment;
use crate::common::{make_request, resolve_alias, RequestType};
use crate::output::Report;

#[derive(Debug, Parser)]
#[command(about = "Grant permissions to a member in a context")]
pub struct GrantPermissionCommand {
    #[arg(help = "The context ID")]
    #[arg(long, short, default_value = "default")]
    pub context: Alias<ContextId>,

    #[arg(help = "The granter's public key")]
    #[arg(long = "as", default_value = "default")]
    pub granter: Alias<PublicKey>,

    #[arg(help = "The grantee's public key")]
    pub grantee: Alias<PublicKey>,

    #[arg(help = "The capability to grant")]
    #[clap(value_enum)]
    pub capability: Capability,
}

impl GrantPermissionCommand {
    pub async fn run(self, environment: &Environment) -> eyre::Result<()> {
        let connection = environment
            .connection
            .as_ref()
            .ok_or_eyre("No connection configured")?;

        let context_id = resolve_alias(
            &connection.api_url,
            connection.auth_key.as_ref(),
            self.context,
            None,
        )
        .await?
        .value()
        .copied()
        .ok_or_eyre("unable to resolve context")?;

        let grantee_id = resolve_alias(
            &connection.api_url,
            connection.auth_key.as_ref(),
            self.grantee,
            Some(context_id),
        )
        .await?
        .value()
        .cloned()
        .ok_or_eyre("unable to resolve grantee identity")?;

        let mut url = connection.api_url.clone();

        url.set_path(&format!(
            "admin-api/dev/contexts/{}/capabilities/grant",
            context_id
        ));

        let request: Vec<(PublicKey, ConfigCapability)> =
            vec![(grantee_id, self.capability.into())];

        make_request::<_, GrantPermissionResponse>(
            environment,
            &Client::new(),
            url,
            Some(request),
            connection.auth_key.as_ref(),
            RequestType::Post,
        )
        .await
    }
}

impl Report for GrantPermissionResponse {
    fn report(&self) {
        println!("Permission granted successfully");
    }
}
