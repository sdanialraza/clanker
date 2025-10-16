use anyhow::{Error, Result};
use dashmap::DashMap;
use openai_api_rust::chat::ChatBody;
use serenity::all::{
	ButtonStyle, CommandInteraction, ComponentInteraction, Context, CreateAllowedMentions, CreateButton,
	CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EventHandler, Interaction, Message,
	MessageFlags, Ready, UserId,
};

use crate::openai;

#[derive(Default)]
pub struct Handler {
	pub history: DashMap<UserId, ChatBody>,
}

impl Handler {
	async fn command_create(&self, ctx: &Context, command: &CommandInteraction) -> Result<()> {
		let option = command
			.data
			.options
			.first()
			.ok_or_else(|| Error::msg("No command options present"))?;

		if option.name == "all" {
			let application = ctx.http.get_current_application_info().await?;

			if application.owner.is_none_or(|owner| command.user.id != owner.id) {
				anyhow::bail!("You are not the application owner");
			}

			self.history.clear();

			let message = CreateInteractionResponseMessage::new().content("Cleared all chat histories!");
			let response = CreateInteractionResponse::Message(message);

			command.create_response(ctx, response).await?;
		}

		if option.name == "history" {
			self.history.remove(&command.user.id);

			let message = CreateInteractionResponseMessage::new().content("Cleared your chat history!");
			let response = CreateInteractionResponse::Message(message);

			command.create_response(ctx, response).await?;
		}

		Ok(())
	}

	async fn component_create(&self, ctx: &Context, component: &ComponentInteraction) -> Result<()> {
		if component.user.id.to_string() != component.data.custom_id {
			anyhow::bail!("Clanker did not reply to you");
		}

		component.message.delete(ctx).await?;

		Ok(())
	}

	async fn message_create(&self, ctx: &Context, message: &Message) -> Result<()> {
		let mut body = self.history.entry(message.author.id).or_insert(openai::body()?);
		let reply = message.referenced_message.as_ref().map(|msg| msg.content.as_str());

		openai::post(body.value_mut(), message.content.clone(), reply)?;

		let button = CreateButton::new(message.author.id.to_string())
			.label("Delete")
			.style(ButtonStyle::Danger);

		let builder = CreateMessage::new()
			.allowed_mentions(CreateAllowedMentions::new())
			.button(button)
			.content(body.messages.last().unwrap().content.clone())
			.flags(MessageFlags::SUPPRESS_EMBEDS)
			.reference_message(message);

		message.channel_id.send_message(ctx, builder).await?;

		Ok(())
	}
}

#[serenity::async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		let result = match &interaction {
			Interaction::Command(command) => self.command_create(&ctx, command).await,
			Interaction::Component(component) => self.component_create(&ctx, component).await,
			_ => return,
		};

		if let Err(error) = result {
			let message = CreateInteractionResponseMessage::new()
				.content(format!(":no_entry_sign: {error}!"))
				.ephemeral(true);

			let response = CreateInteractionResponse::Message(message);

			let result = match &interaction {
				Interaction::Command(command) => command.create_response(&ctx, response).await,
				Interaction::Component(component) => component.create_response(&ctx, response).await,
				_ => return,
			};

			if result.is_err() {
				eprintln!("An error occurred: {error}");
			}
		}
	}

	async fn message(&self, ctx: Context, message: Message) {
		if message.author.bot && message.webhook_id.is_none() {
			return;
		}

		let firsts = ["btw", "hello", "hey", "hi", "oi", "ok", "okay", "so", "sup", "wtf"];
		let seconds = ["bot", "clank", "clanka", "clanker", "google", "gpt", "grok", "siri"];

		let lower = message.content.to_lowercase();
		let mut words = lower.split([' ', ',']).filter(|word| !word.is_empty());

		if words.next().is_none_or(|word| !firsts.contains(&word)) {
			return;
		}

		if words.next().is_none_or(|word| !seconds.contains(&word)) {
			return;
		}

		if let Err(error) = self.message_create(&ctx, &message).await {
			let button = CreateButton::new(message.author.id.to_string())
				.label("Delete")
				.style(ButtonStyle::Danger);

			let builder = CreateMessage::new()
				.allowed_mentions(CreateAllowedMentions::new())
				.button(button)
				.content(format!(":no_entry_sign: {error}!"))
				.flags(MessageFlags::SUPPRESS_EMBEDS)
				.reference_message(&message);

			if message.channel_id.send_message(&ctx, builder).await.is_err() {
				eprintln!("An error occurred: {error}");
			}
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is running!", ready.user.name);
	}
}
