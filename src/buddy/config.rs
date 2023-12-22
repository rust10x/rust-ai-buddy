use crate::ais::asst;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Config {
	pub name: String,
	pub model: String,
	pub instructions_file: String,
	pub file_bundles: Vec<FileBundle>,
}

#[derive(Debug, Deserialize)]
pub(super) struct FileBundle {
	pub bundle_name: String,
	pub src_dir: String,
	pub dst_ext: String,
	pub src_globs: Vec<String>,
}

// region:    --- Froms

impl From<&Config> for asst::CreateConfig {
	fn from(config: &Config) -> Self {
		Self {
			name: config.name.clone(),
			model: config.model.clone(),
		}
	}
}

// endregion: --- Froms
