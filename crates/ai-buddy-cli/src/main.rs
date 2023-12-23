// region:    --- Modules

mod error;
mod utils;

pub use self::error::{Error, Result};
use crate::utils::cli::{
	ico_check, ico_deleted_ok, ico_err, ico_uploaded, ico_uploading,
};
use ai_buddy::event::{AisEvent, Event, EventBus};
use ai_buddy::{Buddy, BuddyEvent};
use console::Term;
use std::io::{self, Write};
use std::time::Duration;
use textwrap::wrap;
use tokio::time::sleep;
use utils::cli::{ico_res, prompt, txt_res};

// endregion: --- Modules

#[tokio::main]
async fn main() {
	println!();
	let _ = io::stdout().flush();

	match start().await {
		Ok(_) => println!("\nBye!\n"),
		Err(e) => println!("\nError: {}\n", e),
	}
}

const DEFAULT_DIR: &str = "buddy";

// region:    --- Types

/// Input Command from the user
#[derive(Debug)]
enum Cmd {
	Quit,
	Chat(String),
	RefreshAll,
	RefreshConv,
	RefreshInst,
	RefreshFiles,
}

impl Cmd {
	fn from_input(input: impl Into<String>) -> Self {
		let input = input.into();

		if input == "/q" {
			Self::Quit
		} else if input == "/r" || input == "/ra" {
			Self::RefreshAll
		} else if input == "/ri" {
			Self::RefreshInst
		} else if input == "/rf" {
			Self::RefreshFiles
		} else if input == "/rc" {
			Self::RefreshConv
		} else {
			Self::Chat(input)
		}
	}
}

// endregion: --- Types

async fn start() -> Result<()> {
	let event_bus = EventBus::new();

	let _ = event_printer(&event_bus).await;

	let mut buddy =
		Buddy::init_from_dir(DEFAULT_DIR, false, Some(event_bus)).await?;

	let mut conv = buddy.load_or_create_conv(false).await?;

	loop {
		// TODO: This sleep needs to be removed.
		//       It is a workaround to ensure that the event_printer and the "Ask away" functionality work correctly.
		//       Eventually, we need to implement a "buddy.ready()" scheme or something similar.
		sleep(Duration::from_millis(50)).await;

		let input = prompt("Ask away")?;

		let cmd = Cmd::from_input(input);

		match cmd {
			Cmd::Quit => break,

			Cmd::Chat(msg) => {
				let res = buddy.chat(&conv, &msg).await?;
				let res = wrap(&res, 80).join("\n");
				println!("{} {}", ico_res(), txt_res(res));
			}

			Cmd::RefreshAll => {
				let event_bus = EventBus::new();
				let _ = event_printer(&event_bus).await;

				buddy =
					Buddy::init_from_dir(DEFAULT_DIR, true, Some(event_bus)).await?;
				conv = buddy.load_or_create_conv(true).await?;
			}

			Cmd::RefreshConv => {
				conv = buddy.load_or_create_conv(true).await?;
			}

			Cmd::RefreshInst => {
				buddy.upload_instructions().await?;
				conv = buddy.load_or_create_conv(true).await?;
			}

			Cmd::RefreshFiles => {
				buddy.upload_files(true).await?;
				conv = buddy.load_or_create_conv(true).await?;
			}
		}
	}

	Ok(())
}

async fn event_printer(event_bus: &EventBus) -> Result<()> {
	let mut rx = event_bus.subscribe()?;

	tokio::spawn(async move {
		let term = Term::stdout();

		loop {
			let evt = rx.recv().await;
			let _ = term.flush();

			if let Ok(evt) = evt {
				match evt {
					Event::Ais(ais_evt) => match ais_evt {
						AisEvent::AsstCreated(asst_ref) => {
							let _ = term.write_line(&format!(
								"{} Assistant {} created",
								ico_check(),
								asst_ref.name
							));
						}
						AisEvent::AsstLoaded(asst_ref) => {
							let _ = term.write_line(&format!(
								"{} Assistant {} loaded",
								ico_check(),
								asst_ref.name
							));
						}
						AisEvent::AsstDeleted(asst_ref) => {
							let _ = term.write_line(&format!(
								"{} Assistant {} deleted",
								ico_deleted_ok(),
								asst_ref.name
							));
						}
						AisEvent::OrgFileDeleted(file_ref) => {
							let _ = term.write_line(&format!(
								"{} File {} deleted",
								ico_deleted_ok(),
								file_ref.name
							));
						}
						AisEvent::OrgFileUploading { file_name } => {
							let _ = term.write_line(&format!(
								"{} Uploading {}",
								ico_uploading(),
								file_name
							));
						}
						AisEvent::OrgFileUploaded(file_ref) => {
							let _ = term.write_line(&format!(
								"{} Uploaded  {}",
								ico_uploaded(),
								file_ref.name
							));
						}

						AisEvent::OrgFileCantDelete { file_ref, cause } => {
							let _ = term.write_line(&format!(
								"{} File {} can't be deleted: {}",
								ico_err(),
								file_ref.name,
								cause
							));
						}

						AisEvent::AsstFileCantRemove {
							asst_id,
							file_id,
							cause,
						} => {
							let _ = term.write_line(&format!(
							"{} File {} can't be removed from assistant {}\n   cause: {cause}",
							ico_err(),
							file_id,
							asst_id
						));
						}
					},

					Event::Buddy(buddy_event) => match buddy_event {
						BuddyEvent::InstUploaded => {
							let _ = term.write_line(&format!(
								"{} Instructions uploaded",
								ico_check()
							));
						}
						BuddyEvent::ConvCreated => {
							let _ = term.write_line(&format!(
								"{} Conversation created",
								ico_check()
							));
						}
						BuddyEvent::ConvLoaded => {
							let _ = term.write_line(&format!(
								"{} Conversation loaded",
								ico_check()
							));
						}
					},
				}
			} else {
				// if here, the event_bus has been changed, ok to break, nothing to print.
				break;
			};

			let _ = term.flush();
		}
	});

	Ok(())
}
