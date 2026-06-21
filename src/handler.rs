use anyhow::{Error, Result};
use gemini_rust::{Message as GeminiMessage, Tool};
use serenity::all::{
	ButtonStyle, CommandInteraction, ComponentInteraction, Context, CreateAllowedMentions, CreateButton,
	CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EventHandler, Interaction, Message,
	MessageFlags, Ready, async_trait,
};

use crate::{History, State};

pub struct Handler;

impl Handler {
	async fn command_create(context: &Context, command: &CommandInteraction) -> Result<()> {
		if command.data.name == "clear-history" {
			let mut data = context.data.write().await;

			data.get_mut::<History>()
				.ok_or_else(|| Error::msg("Could not get the chat history!"))?
				.clear();

			let message = CreateInteractionResponseMessage::new().content("Cleared the chat history!");

			let response = CreateInteractionResponse::Message(message);

			command.create_response(context, response).await?;
		}

		Ok(())
	}

	async fn component_create(context: &Context, component: &ComponentInteraction) -> Result<()> {
		if component.user.id.to_string() != component.data.custom_id {
			anyhow::bail!("{} did not reply to you!", context.cache.current_user().name);
		}

		component.message.delete(context).await?;

		Ok(())
	}

	async fn message_create(context: &Context, message: &Message) -> Result<()> {
		message.channel_id.broadcast_typing(context).await?;

		let (gemini, base_prompt, current_history) = {
			let data = context.data.read().await;

			let state = data
				.get::<State>()
				.ok_or_else(|| Error::msg("Could not get the bot state!"))?;

			let history = data
				.get::<History>()
				.ok_or_else(|| Error::msg("Could not get the chat history!"))?;

			(
				state.gemini_client.clone(),
				state.system_prompt.clone(),
				history.clone(),
			)
		};

		let system_prompt = base_prompt
			.replace("$id", &context.cache.current_user().id.to_string())
			.replace("$name", &context.cache.current_user().name)
			.replace("$tag", &context.cache.current_user().tag());

		let google_search_tool = Tool::google_search();

		let author_name = message
			.author
			.global_name
			.clone()
			.unwrap_or_else(|| message.author.name.clone());

		let message_content = format!("{}: {}", author_name, message.content);

		let response = gemini
			.generate_content()
			.with_system_instruction(system_prompt)
			.with_messages(current_history)
			.with_user_message(&message_content)
			.with_tool(google_search_tool)
			.execute()
			.await?
			.text();

		let response = if response.trim().is_empty() {
			"-# (empty)".into()
		} else {
			response
		};

		let button = CreateButton::new(message.author.id.to_string())
			.label("Delete")
			.style(ButtonStyle::Danger);

		let builder = CreateMessage::new()
			.allowed_mentions(CreateAllowedMentions::new())
			.button(button)
			.content(&response)
			.flags(MessageFlags::SUPPRESS_EMBEDS)
			.reference_message(message);

		message.channel_id.send_message(context, builder).await?;

		{
			let mut data = context.data.write().await;

			let history = data
				.get_mut::<History>()
				.ok_or_else(|| Error::msg("Could not get histories!"))?;

			history.push(GeminiMessage::user(message_content));

			history.push(GeminiMessage::model(response));
		}

		Ok(())
	}
}

#[async_trait]

impl EventHandler for Handler {
	async fn interaction_create(&self, context: Context, interaction: Interaction) {
		let result = match &interaction {
			Interaction::Command(command) => Self::command_create(&context, command).await,

			Interaction::Component(component) => Self::component_create(&context, component).await,

			_ => return,
		};

		if let Err(error) = result {
			let message = CreateInteractionResponseMessage::new()
				.content(format!(":no_entry_sign: {error}"))
				.ephemeral(true);

			let response = CreateInteractionResponse::Message(message);

			let result = match &interaction {
				Interaction::Command(command) => command.create_response(&context, response).await,

				Interaction::Component(component) => component.create_response(&context, response).await,

				_ => return,
			};

			if result.is_err() {
				eprintln!("An error occurred: {error}");
			}
		}
	}

	async fn message(&self, context: Context, message: Message) {
		if message.author.bot && message.webhook_id.is_none() {
			return;
		}

		if !message.mentions_user_id(context.cache.current_user().id) {
			let greetings = ["hello", "hey", "hi"];

			let lower = message.content.to_lowercase();

			let mut words = lower
				.split_whitespace()
				.map(|word| word.strip_suffix(',').unwrap_or(word));

			if words.next().is_none_or(|word| !greetings.contains(&word)) {
				return;
			}

			if words.next() != Some(&context.cache.current_user().name.to_lowercase()) {
				return;
			}
		}

		if let Err(error) = Self::message_create(&context, &message).await {
			let button = CreateButton::new(message.author.id.to_string())
				.label("Delete")
				.style(ButtonStyle::Danger);

			let builder = CreateMessage::new()
				.allowed_mentions(CreateAllowedMentions::new())
				.button(button)
				.content(format!(":no_entry_sign: {error}"))
				.flags(MessageFlags::SUPPRESS_EMBEDS)
				.reference_message(&message);

			if message.channel_id.send_message(&context, builder).await.is_err() {
				eprintln!("An error occurred: {error}");
			}
		}
	}

	async fn ready(&self, _context: Context, ready: Ready) {
		println!("{} is running!", ready.user.name);
	}
}
