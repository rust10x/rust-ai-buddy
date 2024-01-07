use crate::event;
use async_openai::error::OpenAIError;
use async_openai::types::RunStatus;
use derive_more::From;
use std::io;
use tokio::sync::broadcast;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
	// -- buddy
	ShouldNotDeleteLocalFile(String),
	CannotFindThreadIdForConv(String),

	// -- ais
	MessageImageNotSupported,
	NoMessageInMessageObjectContent,
	NoMessageFoundInMessages,
	NoOpenAIApiKeyInEnv,
	DeleteAllFilesRequiresAtLeastOneGlob,
	RunError(RunStatus),

	// -- Event
	#[from]
	BoadcastSend(broadcast::error::SendError<event::Event>),

	// -- Std
	#[from]
	IO(io::Error),

	// -- Externals
	#[from]
	SimpleFs(simple_fs::Error),
	#[from]
	OpenAI(OpenAIError),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
