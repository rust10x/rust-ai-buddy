pub use crate::ais::AisEvent;
pub use crate::buddy::BuddyEvent;

use crate::Result;
use derive_more::From;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Debug, Clone, From)]
pub enum Event {
	Ais(AisEvent),
	Buddy(BuddyEvent),
}

/// EventBus allows all the components of this crate to send their events
/// so that other services can subscribe to them.
///
/// Notes:
/// - This is a clone-efficient structure, so it's okay to be cloned and owned.
/// - Currently, it uses a Tokio broadcast channel, but this implementation detail is hidden behind the API.
/// - `_rx` is the Receiver and is kept in an Arc to prevent the channel from closing. It is not clonable.
#[derive(Debug, Clone)]
pub struct EventBus {
	tx: Sender<Event>,
	_rx: Arc<Receiver<Event>>,
}

impl EventBus {
	#[allow(clippy::new_without_default)]
	pub fn new() -> EventBus {
		let (tx, rx) = broadcast::channel::<Event>(16);
		EventBus {
			tx,
			_rx: Arc::new(rx),
		}
	}

	pub(crate) fn send(&self, evt: impl Into<Event>) -> Result<()> {
		let evt = evt.into();
		self.tx.send(evt)?;
		Ok(())
	}

	pub fn subscribe(&self) -> Result<Receiver<Event>> {
		Ok(self.tx.subscribe())
	}
}
