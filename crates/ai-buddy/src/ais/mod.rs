//! The `ais` module is designed to be the interface with specific AI services, such as OpenAI.
//!
//! Currently, it is a mono-provider implementation with OpenAI only, but the goal
//! is to be a multi-provider, supporting ollama, lamafile, gemini, etc...
//!
//! Currently, it is mostly designed as an assistant interface, but this might or might not change over time.

// region:    --- Modules

pub mod asst;
mod event;
pub mod msg;
mod types;

pub use event::AisEvent;
pub use types::*;

use crate::event::EventBus;
use crate::{Error, Result};
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use simple_fs::get_glob_set;

// endregion: --- Modules

// region:    --- Client

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

pub type OaClient = Client<OpenAIConfig>;

/// Wraps the async-openai client and provides additional functionalities
/// such as an event bus.
#[derive(Debug)]
pub struct AisClient {
	oa_client: OaClient,
	event_bus: EventBus,
}

impl AisClient {
	pub fn oa_client(&self) -> &OaClient {
		&self.oa_client
	}
	pub fn event_bus(&self) -> &EventBus {
		&self.event_bus
	}
}

pub fn new_ais_client(event_bus: EventBus) -> Result<AisClient> {
	if std::env::var(ENV_OPENAI_API_KEY).is_ok() {
		Ok(AisClient {
			oa_client: Client::new(),
			event_bus,
		})
	} else {
		println!("No {ENV_OPENAI_API_KEY} env variable. Please set it.");

		Err(Error::NoOpenAIApiKeyInEnv)
	}
}

// endregion: --- Client

// region:    --- Danger Zone

// DANGER ZONE - Make sure to triple check before calling. Not pub for now.
#[allow(dead_code)]
async fn delete_org_files(oac: &OaClient, globs: &[&str]) -> Result<u32> {
	let oa_files = oac.files();
	let files = oa_files.list(&[("purpose", "assistants")]).await?;
	let mut count = 0;

	if globs.is_empty() {
		return Err(Error::DeleteAllFilesRequiresAtLeastOneGlob);
	}

	let globs = get_glob_set(globs)?;

	for file in files.data {
		count += 1;
		if globs.is_match(&file.filename) {
			oa_files.delete(&file.id).await?;
			println!("DELETED: {:?}", file.filename);
		} else {
			println!("DELETE SKIPPED: {:?}", file.filename);
		}
	}

	Ok(count)
}

// endregion: --- Danger Zone
