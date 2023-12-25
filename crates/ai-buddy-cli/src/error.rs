use derive_more::From;
use std::io;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
	// -- App Libs
	#[from]
	AIBuddy(ai_buddy::Error),

	// -- Externals
	#[from]
	IO(io::Error),
	#[from]
	Dialoguer(dialoguer::Error),
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
