use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

// region:    --- Asst

#[derive(Debug, Clone, From, Deref, Display)]
pub struct AsstId(String);

impl From<&AsstId> for AsstId {
	fn from(val: &AsstId) -> Self {
		val.clone()
	}
}

#[derive(Debug, Clone)]
pub struct AsstRef {
	pub name: String,
	pub id: AsstId,
}

impl AsstRef {
	pub fn new(name: impl Into<String>, id: AsstId) -> Self {
		Self {
			name: name.into(),
			id,
		}
	}
}

// endregion: --- Asst

// region:    --- File

#[derive(Debug, Clone, From, Deref, Display)]
pub struct FileId(String);

#[derive(Debug, Clone)]
pub struct FileRef {
	pub name: String,
	pub id: FileId,
}

impl FileRef {
	pub fn new(name: impl Into<String>, id: FileId) -> Self {
		Self {
			name: name.into(),
			id,
		}
	}
}

// endregion: --- File

// region:    --- ThreadId

#[derive(Debug, From, Deref, Display, Serialize, Deserialize)]
pub struct ThreadId(String);

// endregion: --- ThreadId
