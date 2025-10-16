mod handler;
mod openai;

use std::env;

use anyhow::Result;
use serenity::all::{ActivityData, Client, GatewayIntents};

use crate::handler::Handler;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv()?;

	let token = env::var("DISCORD_TOKEN")?;
	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(token, intents)
		.activity(ActivityData::listening("your prompts"))
		.event_handler(Handler::default())
		.await?;

	client.start().await?;

	Ok(())
}
