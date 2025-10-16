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
	async fn command_create(&self, ctx: Context, command: CommandInteraction) {
		let subcommand = command.data.options.first().unwrap().name.as_str();

		if subcommand == "all" {
			let application = ctx.http.get_current_application_info().await.unwrap();

			if command.user.id != application.owner.unwrap().id {
				let message = CreateInteractionResponseMessage::new()
					.content("You are not the application owner!")
					.ephemeral(true);

				let response = CreateInteractionResponse::Message(message);

				command.create_response(&ctx, response).await.unwrap();
			} else {
				self.history.clear();

				let message = CreateInteractionResponseMessage::new().content("Cleared all chat histories!");
				let response = CreateInteractionResponse::Message(message);

				command.create_response(&ctx, response).await.unwrap();
			}
		}

		if subcommand == "history" {
			self.history.remove(&command.user.id);

			let message = CreateInteractionResponseMessage::new().content("Cleared your chat history!");
			let response = CreateInteractionResponse::Message(message);

			command.create_response(&ctx, response).await.unwrap();
		}
	}

	async fn component_create(&self, ctx: Context, component: ComponentInteraction) {
		if component.user.id.to_string() == component.data.custom_id {
			component.message.delete(&ctx).await.unwrap();
			return;
		}

		let message = CreateInteractionResponseMessage::new()
			.content("Clanker did not reply to you!")
			.ephemeral(true);

		let response = CreateInteractionResponse::Message(message);

		component.create_response(&ctx, response).await.unwrap();
	}
}

#[serenity::async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		match interaction {
			Interaction::Command(command) => self.command_create(ctx, command).await,
			Interaction::Component(component) => self.component_create(ctx, component).await,
			_ => (),
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

		let mut body = self.history.entry(message.author.id).or_insert(openai::body());
		let reply = message.referenced_message.as_ref().map(|msg| msg.content.as_str());

		openai::post(body.value_mut(), message.content.clone(), reply);

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
