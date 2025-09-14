use serenity::all::{
	ButtonStyle, Context, CreateAllowedMentions, CreateButton, CreateInteractionResponse,
	CreateInteractionResponseMessage, CreateMessage, EventHandler, Interaction, Message, MessageFlags, Ready,
};

use crate::{History, openai};

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		let Some(component) = interaction.as_message_component() else {
			return;
		};

		let mut response = CreateInteractionResponse::Acknowledge;

		if component.user.id.to_string() == component.data.custom_id {
			component.message.delete(&ctx).await.unwrap();
		} else {
			let message = CreateInteractionResponseMessage::new()
				.content("Clanker did not reply to you!")
				.ephemeral(true);

			response = CreateInteractionResponse::Message(message);
		}

		component.create_response(&ctx, response).await.unwrap();
	}

	async fn message(&self, ctx: Context, message: Message) {
		if message.author.bot {
			return;
		}

		let firsts = ["hello", "hey", "hi", "oi", "ok", "sup"];
		let seconds = ["bot", "clanka", "clanker", "google", "gpt", "siri"];

		let lower = message.content.to_lowercase();
		let mut split = lower.split_whitespace();

		if !split.next().is_some_and(|x| firsts.contains(&x)) {
			return;
		}

		if !split.next().is_some_and(|x| seconds.contains(&x)) {
			return;
		}

		let data = ctx.data.read().await;
		let history = data.get::<History>().unwrap();

		let mut body = history.entry(message.author.id).or_insert(openai::body());

		openai::post(body.value_mut(), message.content.clone());

		let button = CreateButton::new(message.author.id.to_string())
			.label("Delete")
			.style(ButtonStyle::Danger);

		let builder = CreateMessage::new()
			.allowed_mentions(CreateAllowedMentions::new())
			.button(button)
			.content(body.messages.last().unwrap().content.clone())
			.flags(MessageFlags::SUPPRESS_EMBEDS)
			.reference_message(&message);

		message.channel_id.send_message(&ctx, builder).await.unwrap();
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is running!", ready.user.name);
	}
}
