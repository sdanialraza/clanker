use std::env::var;

use anyhow::Result;
use dotenvy::dotenv;
use serenity::all::{CreateCommand, HttpBuilder, InteractionContext};
use tokio::main;

#[main]

async fn main() -> Result<()> {
	let _ = dotenv();

	let application_id = var("DISCORD_APPLICATION_ID")?.parse()?;
	let token = var("DISCORD_TOKEN")?;

	let http = HttpBuilder::new(token).application_id(application_id).build();

	let clear = CreateCommand::new("clear-history")
		.add_context(InteractionContext::Guild)
		.description("Clears the chat history");

	http.create_global_commands(&[clear]).await?;

	Ok(())
}
