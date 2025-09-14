use std::env::var;

use openai_api_rust::chat::*;
use openai_api_rust::*;
use serenity::all::{Context, EventHandler, Message as DiscordMessage, Ready};

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, message: DiscordMessage) {
		if message.author.bot {
			return;
		}

		let base_url = var("OPENAI_BASE_URL").unwrap();

		if message.content.to_lowercase().starts_with("hey clanker")
			|| message.content.to_lowercase().starts_with("hey clanka")
		{
			let auth = Auth::from_env().unwrap();
			let openai = OpenAI::new(auth, base_url.as_str());
			let body = ChatBody {
				model: "gpt-4.1-nano".to_string(),
				max_tokens: Some(500),
				temperature: Some(0_f32),
				top_p: Some(0_f32),
				n: Some(1),
				stream: Some(false),
				stop: None,
				presence_penalty: None,
				frequency_penalty: None,
				logit_bias: None,
				user: None,
				messages: vec![
					Message {
						content: "You are Clanker, a helpful and friendly AI assistant. Always respond in a concise and clear manner. If you don't know the answer, admit it honestly.".to_string(),
						role: Role::System,
					},
					Message {
						content: message.content.clone(),
						role: Role::System,
					},
				],
			};

			let rs = openai.chat_completion_create(&body);
			let choice = rs.unwrap().choices;
			let response = choice[0].message.as_ref().unwrap();

			message.reply(ctx, response.content.clone()).await.unwrap();
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is running!", ready.user.name);
	}
}
