mod handler;
mod openai;

use std::env;

use dashmap::DashMap;
use openai_api_rust::chat::ChatBody;
use serenity::all::{ActivityData, Client, GatewayIntents, UserId};
use serenity::prelude::TypeMapKey;

use crate::handler::Handler;

struct History;

impl TypeMapKey for History {
	type Value = DashMap<UserId, ChatBody>;
}

#[tokio::main]
async fn main() {
	dotenvy::dotenv().unwrap();

	let token = env::var("DISCORD_TOKEN").unwrap();
	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(token, intents)
		.activity(ActivityData::custom("dirty clanker"))
		.event_handler(Handler)
		.type_map_insert::<History>(DashMap::new())
		.await
		.unwrap();

	client.start().await.unwrap();
}
