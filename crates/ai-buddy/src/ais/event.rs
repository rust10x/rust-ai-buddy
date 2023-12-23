//! Ais Event

use crate::ais::{AsstId, AsstRef, FileId, FileRef};

#[derive(Debug, Clone)]
pub enum AisEvent {
	// -- Asst Events
	AsstCreated(AsstRef),
	AsstLoaded(AsstRef),
	AsstDeleted(AsstRef),
	AsstFileCantRemove {
		asst_id: AsstId,
		file_id: FileId,
		cause: String,
	},

	// -- File Events
	OrgFileUploading {
		file_name: String,
	},
	OrgFileUploaded(FileRef),

	OrgFileDeleted(FileRef),
	OrgFileCantDelete {
		file_ref: FileRef,
		cause: String,
	},
}
