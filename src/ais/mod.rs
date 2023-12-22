// region:    --- Modules

pub mod asst;
pub mod msg;

use crate::utils::files::get_glob_set;
use crate::Result;
use async_openai::config::OpenAIConfig;
use async_openai::Client;

// endregion: --- Modules

// region:    --- Client

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

pub type OaClient = Client<OpenAIConfig>;

pub fn new_oa_client() -> Result<OaClient> {
	if std::env::var(ENV_OPENAI_API_KEY).is_ok() {
		Ok(Client::new())
	} else {
		println!("No {ENV_OPENAI_API_KEY} env variable. Please set it.");

		Err("No openai api key in env".into())
	}
}

// endregion: --- Client

// region:    --- Danger Zone

// DANGER ZONE - Make sure to triple check before calling. Not pub for now.
#[allow(dead_code)]
async fn delete_org_files(oac: &OaClient, globs: &[&str]) -> Result<u32> {
	let oa_files = oac.files();
	let files = oa_files.list().await?;
	let mut count = 0;

	if globs.is_empty() {
		return Err("asst::delete_all_files requires at least one glob".into());
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
