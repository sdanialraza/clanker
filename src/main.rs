mod handler;
mod openai;

use std::collections::HashMap;
use std::env;

use anyhow::Result;
use openai_api_rust::chat::ChatBody;
use serenity::all::{ActivityData, Client, GatewayIntents, UserId};
use serenity::prelude::TypeMapKey;

use crate::handler::Handler;

pub struct History;

impl TypeMapKey for History {
	type Value = HashMap<UserId, ChatBody>;
}

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv()?;

	let token = env::var("DISCORD_TOKEN")?;
	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(token, intents)
		.activity(ActivityData::listening("your prompts"))
		.event_handler(Handler)
		.await?;

	client.start().await?;

	Ok(())
}
