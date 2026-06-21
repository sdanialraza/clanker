use std::env::var;

use anyhow::Result;
use dotenvy::dotenv;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, HttpBuilder, InteractionContext};
use tokio::main;

#[main]

async fn main() -> Result<()> {
	dotenv()?;

	let application_id = var("DISCORD_APPLICATION_ID")?.parse()?;
	let token = var("DISCORD_TOKEN")?;

	let http = HttpBuilder::new(token).application_id(application_id).build();

	let history = CreateCommandOption::new(CommandOptionType::SubCommand, "history", "Clears this server's history");

	let clear = CreateCommand::new("clear")
		.add_context(InteractionContext::Guild)
		.description("Commands for clearing chat histories")
		.set_options(vec![history]);

	http.create_global_commands(&[clear]).await?;

	Ok(())
}
