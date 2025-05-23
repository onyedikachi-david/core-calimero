use calimero_primitives::alias::Alias;
use calimero_primitives::context::ContextId;
use calimero_primitives::identity::PublicKey;
use calimero_server_primitives::admin::GetContextIdentitiesResponse;
use clap::Parser;
use eyre::{OptionExt, Result as EyreResult};
use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use reqwest::Client;

use crate::cli::Environment;
use crate::common::{
    create_alias, fetch_multiaddr, load_config, make_request, multiaddr_to_url, resolve_alias,
    RequestType,
};

mod alias;
mod generate;

#[derive(Debug, Parser)]
#[command(about = "Manage context identities")]
pub struct ContextIdentityCommand {
    #[command(subcommand)]
    command: ContextIdentitySubcommand,
}

#[derive(Debug, Parser)]
pub enum ContextIdentitySubcommand {
    #[command(about = "List identities in a context", alias = "ls")]
    List {
        #[arg(help = "The context whose identities we're listing")]
        #[arg(long, short, default_value = "default")]
        context: Alias<ContextId>,
        #[arg(long, help = "Show only owned identities")]
        owned: bool,
    },
    #[command(about = "Manage identity aliases")]
    Alias(alias::ContextIdentityAliasCommand),
    #[command(about = "Generate a new identity keypair", alias = "new")]
    Generate(generate::GenerateCommand),
    #[command(about = "Set default identity for a context")]
    Use {
        #[arg(help = "The identity to set as default")]
        identity: PublicKey,
        #[arg(help = "The context to set the identity for")]
        #[arg(long, short, default_value = "default")]
        context: Alias<ContextId>,
    },
}

impl ContextIdentityCommand {
    pub async fn run(self, environment: &Environment) -> EyreResult<()> {
        let config = load_config(&environment.args.home, &environment.args.node_name)?;
        let multiaddr = fetch_multiaddr(&config)?;
        let client = Client::new();

        match self.command {
            ContextIdentitySubcommand::List { context, owned } => {
                list_identities(
                    environment,
                    &multiaddr,
                    &client,
                    &config.identity,
                    Some(context),
                    owned,
                )
                .await
            }
            ContextIdentitySubcommand::Alias(cmd) => cmd.run(environment).await,
            ContextIdentitySubcommand::Generate(cmd) => cmd.run(environment).await,
            ContextIdentitySubcommand::Use { identity, context } => {
                let resolve_response =
                    resolve_alias(multiaddr, &config.identity, context, None).await?;

                let context_id = resolve_response
                    .value()
                    .cloned()
                    .ok_or_eyre("Failed to resolve context: no value found")?;
                let default_alias: Alias<PublicKey> =
                    "default".parse().expect("'default' is a valid alias name");

                let res = create_alias(
                    multiaddr,
                    &config.identity,
                    default_alias,
                    Some(context_id),
                    identity,
                )
                .await?;

                environment.output.write(&res);
                Ok(())
            }
        }
    }
}

async fn list_identities(
    environment: &Environment,
    multiaddr: &Multiaddr,
    client: &Client,
    keypair: &Keypair,
    context: Option<Alias<ContextId>>,
    owned: bool,
) -> EyreResult<()> {
    let resolve_response = resolve_alias(
        multiaddr,
        keypair,
        context.unwrap_or_else(|| "default".parse().expect("valid alias")),
        None,
    )
    .await?;

    let context_id = resolve_response
        .value()
        .cloned()
        .ok_or_eyre("Failed to resolve context: no value found")?;

    let endpoint = if owned {
        format!("admin-api/dev/contexts/{}/identities-owned", context_id)
    } else {
        format!("admin-api/dev/contexts/{}/identities", context_id)
    };

    let url = multiaddr_to_url(multiaddr, &endpoint)?;
    make_request::<_, GetContextIdentitiesResponse>(
        environment,
        client,
        url,
        None::<()>,
        keypair,
        RequestType::Get,
    )
    .await
}
