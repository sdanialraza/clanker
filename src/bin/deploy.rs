use std::env;

use anyhow::Result;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, HttpBuilder, InteractionContext};

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv()?;

	let application_id = env::var("DISCORD_APPLICATION_ID")?.parse()?;
	let token = env::var("DISCORD_TOKEN")?;
	let http = HttpBuilder::new(token).application_id(application_id).build();

	let all = CreateCommandOption::new(CommandOptionType::SubCommand, "all", "Clears all servers' histories");
	let history = CreateCommandOption::new(CommandOptionType::SubCommand, "history", "Clears this server's history");

	let clear = CreateCommand::new("clear")
		.add_context(InteractionContext::Guild)
		.description("Commands for clearing chat histories")
		.set_options(vec![all, history]);

	http.create_global_commands(&[clear]).await?;

	Ok(())
}
