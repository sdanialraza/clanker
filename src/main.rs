mod handler;
mod model;
mod openai;

use std::collections::HashMap;
use std::env;

use anyhow::Result;
use serenity::all::{ActivityData, Client, GatewayIntents, GuildId};
use serenity::prelude::TypeMapKey;

use crate::handler::Handler;
use crate::model::RequestBody;

struct History;

impl TypeMapKey for History {
	type Value = HashMap<GuildId, RequestBody>;
}

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv()?;

	let token = env::var("DISCORD_TOKEN")?;
	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(token, intents)
		.activity(ActivityData::custom("Awaiting your prompts"))
		.event_handler(Handler)
		.type_map_insert::<History>(HashMap::new())
		.await?;

	client.start().await?;

	Ok(())
}
