use crate::ais::msg::{get_text_content, user_msg};
use crate::ais::{AisClient, AisEvent, AsstId, AsstRef, FileId, FileRef, ThreadId};
use crate::{Error, Result};
use async_openai::types::{
	AssistantObject, AssistantToolsRetrieval, CreateAssistantFileRequest,
	CreateAssistantRequest, CreateFileRequest, CreateRunRequest,
	CreateThreadRequest, ModifyAssistantRequest, RunStatus, ThreadObject,
};
use console::Term;
use simple_fs::SPath;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time::sleep;

// region:    --- Constants

const DEFAULT_QUERY: &[(&str, &str)] = &[("limit", "100")];
const POLLING_DURATION_MS: u64 = 500;

// endregion: --- Constants

// region:    --- Types

pub struct CreateConfig {
	pub name: String,
	pub model: String,
}

// endregion: --- Types

// region:    --- Asst CRUD

pub async fn create(ais: &AisClient, config: &CreateConfig) -> Result<AsstId> {
	let oac = ais.oa_client();

	let oa_assts = oac.assistants();

	let asst_obj = oa_assts
		.create(CreateAssistantRequest {
			model: config.model.clone(),
			name: Some(config.name.clone()),
			tools: Some(vec![AssistantToolsRetrieval::default().into()]),
			..Default::default()
		})
		.await?;

	let asst_id: AsstId = asst_obj.id.into();

	ais.event_bus().send(AisEvent::AsstCreated(AsstRef::new(
		&config.name,
		asst_id.clone(),
	)))?;

	Ok(asst_id)
}

pub async fn load_or_create(
	ais: &AisClient,
	config: CreateConfig,
	recreate: bool,
) -> Result<AsstId> {
	let asst_obj = first_by_name(ais, &config.name).await?;
	let mut asst_id = asst_obj.map(|o| AsstId::from(o.id));

	// -- Delete asst if recreate true and asst_id
	if let (true, Some(asst_id_ref)) = (recreate, asst_id.as_ref()) {
		delete(ais, asst_id_ref).await?;
		ais.event_bus().send(AisEvent::AsstDeleted(AsstRef::new(
			&config.name,
			asst_id_ref.clone(),
		)))?;
		asst_id.take();
	}

	// -- Create if needed

	if let Some(asst_id) = asst_id {
		ais.event_bus().send(AisEvent::AsstLoaded(AsstRef::new(
			&config.name,
			asst_id.clone(),
		)))?;

		Ok(asst_id)
	} else {
		let asst_id = create(ais, &config).await?;

		Ok(asst_id)
	}
}

pub async fn first_by_name(
	ais: &AisClient,
	name: &str,
) -> Result<Option<AssistantObject>> {
	let oac = ais.oa_client();

	let oa_assts = oac.assistants();

	let assts = oa_assts.list(DEFAULT_QUERY).await?.data;

	let asst_obj = assts
		.into_iter()
		.find(|a| a.name.as_ref().map(|n| n == name).unwrap_or(false));

	Ok(asst_obj)
}

pub async fn upload_instructions(
	ais: &AisClient,
	asst_id: &AsstId,
	inst_content: String,
) -> Result<()> {
	let oac = ais.oa_client();

	let oa_assts = oac.assistants();
	let modif = ModifyAssistantRequest {
		instructions: Some(inst_content),
		..Default::default()
	};
	oa_assts.update(asst_id, modif).await?;

	Ok(())
}

pub async fn delete(ais: &AisClient, asst_id: &AsstId) -> Result<()> {
	let oac = ais.oa_client();

	let oa_assts = oac.assistants();
	let oa_files = oac.files();

	// -- First delete the files associated to this assistant.
	for (file_name, file_id) in get_files_hashmap(ais, asst_id).await?.into_iter() {
		let del_res = oa_files.delete(&file_id).await;
		// NOTE: Might be already deleted, that's ok for now.
		match del_res {
			Ok(_) => ais
				.event_bus()
				.send(AisEvent::OrgFileDeleted(FileRef::new(file_name, file_id)))?,

			Err(err) => ais.event_bus().send(AisEvent::OrgFileCantDelete {
				file_ref: FileRef::new(file_name, file_id),
				cause: err.to_string(),
			})?,
		};
	}

	// Note: No need to delete assistant files since we delete the assistant.

	// -- Delete assistant
	oa_assts.delete(asst_id).await?;

	Ok(())
}

// endregion: --- Asst CRUD

// region:    --- Thread

pub async fn create_thread(ais: &AisClient) -> Result<ThreadId> {
	let oac = ais.oa_client();

	let oa_threads = oac.threads();

	let res = oa_threads
		.create(CreateThreadRequest {
			..Default::default()
		})
		.await?;

	Ok(res.id.into())
}

pub async fn get_thread(
	ais: &AisClient,
	thread_id: &ThreadId,
) -> Result<ThreadObject> {
	let oac = ais.oa_client();

	let oa_threads = oac.threads();

	let thread_obj = oa_threads.retrieve(thread_id).await?;

	Ok(thread_obj)
}

pub async fn run_thread_msg(
	ais: &AisClient,
	asst_id: &AsstId,
	thread_id: &ThreadId,
	msg: &str,
) -> Result<String> {
	let oac = ais.oa_client();

	let msg = user_msg(msg);

	// -- Attach message to thread
	let _message_obj = oac.threads().messages(thread_id).create(msg).await?;

	// -- Create a run for the thread
	let run_request = CreateRunRequest {
		assistant_id: asst_id.to_string(),
		..Default::default()
	};
	let run = oac.threads().runs(thread_id).create(run_request).await?;

	// -- Loop to get result
	let term = Term::stdout();
	loop {
		term.write_str("›")?;
		let run = oac.threads().runs(thread_id).retrieve(&run.id).await?;
		term.write_str("‹ ")?;
		match run.status {
			RunStatus::Completed => {
				term.write_str("\n")?;
				return get_first_thread_msg_content(ais, thread_id).await;
			}
			RunStatus::Queued | RunStatus::InProgress => (),
			other => {
				term.write_str("\n")?;
				return Err(Error::RunError(other));
			}
		}

		sleep(Duration::from_millis(POLLING_DURATION_MS)).await;
	}
}

pub async fn get_first_thread_msg_content(
	ais: &AisClient,
	thread_id: &ThreadId,
) -> Result<String> {
	let oac = ais.oa_client();

	static QUERY: [(&str, &str); 1] = [("limit", "1")];

	let messages = oac.threads().messages(thread_id).list(&QUERY).await?;
	let msg = messages
		.data
		.into_iter()
		.next()
		.ok_or(Error::NoMessageFoundInMessages)?;

	let text = get_text_content(msg)?;

	Ok(text)
}

// endregion: --- Thread

// region:    --- Files

/// Returns the file id by file name hashmap.
pub async fn get_files_hashmap(
	ais: &AisClient,
	asst_id: &AsstId,
) -> Result<HashMap<String, FileId>> {
	let oac = ais.oa_client();

	// -- Get all asst files (files do not have .name)
	let oas_assts = oac.assistants();
	let oa_asst_files = oas_assts.files(asst_id);
	let asst_files = oa_asst_files.list(DEFAULT_QUERY).await?.data;
	let asst_file_ids: HashSet<String> =
		asst_files.into_iter().map(|f| f.id).collect();

	// -- Get all files for org (those files have .filename)
	let oa_files = oac.files();
	let org_files = oa_files.list(&[("purpose", "assistants")]).await?.data;

	// -- Build or file_name:file_id hashmap
	let file_id_by_name: HashMap<String, FileId> = org_files
		.into_iter()
		.filter(|org_file| asst_file_ids.contains(&org_file.id))
		.map(|org_file| (org_file.filename, org_file.id.into()))
		.collect();

	Ok(file_id_by_name)
}

/// Uploads a file to an assistant (first to the account, then attaches to asst)
/// - `force` is `false`, will not upload the file if already uploaded.
/// - `force` is `true`, it will delete existing file (account and asst), and upload.
///
/// Returns `(FileId, has_been_uploaded)`
pub async fn upload_file_by_name(
	ais: &AisClient,
	asst_id: &AsstId,
	file: &SPath,
	force: bool,
) -> Result<(FileId, bool)> {
	let oac = ais.oa_client();

	let file_name = file.file_name();
	let mut file_id_by_name = get_files_hashmap(ais, asst_id).await?;

	let file_id = file_id_by_name.remove(file_name);

	// -- If not force and file already created, return early.
	if !force {
		if let Some(file_id) = file_id {
			return Ok((file_id, false));
		}
	}

	// -- If we have old file_id, we delete the file.
	if let Some(file_id) = file_id {
		// -- Delete the org file
		let oa_files = oac.files();
		if let Err(err) = oa_files.delete(&file_id).await {
			ais.event_bus().send(AisEvent::OrgFileCantDelete {
				file_ref: FileRef::new(file, file_id.clone()),
				cause: err.to_string(),
			})?;
		}

		// -- Delete the asst_file association
		let oa_assts = oac.assistants();
		let oa_assts_files = oa_assts.files(asst_id);
		if let Err(err) = oa_assts_files.delete(&file_id).await {
			ais.event_bus().send(AisEvent::AsstFileCantRemove {
				asst_id: asst_id.clone(),
				file_id: file_id.clone(),
				cause: err.to_string(),
			})?;
		}
	}

	// -- Upload and attach the file.
	ais.event_bus().send(AisEvent::OrgFileUploading {
		file_name: file.file_name().to_string(),
	})?;

	// Upload file.
	let oa_files = oac.files();
	let oa_file = oa_files
		.create(CreateFileRequest {
			file: file.into(),
			purpose: "assistants".into(),
		})
		.await?;

	// Update print.
	ais.event_bus()
		.send(AisEvent::OrgFileUploaded(FileRef::new(
			file,
			oa_file.id.clone().into(),
		)))?;

	// Attach file to assistant.
	let oa_assts = oac.assistants();
	let oa_assts_files = oa_assts.files(asst_id);
	let asst_file_obj = oa_assts_files
		.create(CreateAssistantFileRequest {
			file_id: oa_file.id.clone(),
		})
		.await?;

	// -- Assert warning.
	if oa_file.id != asst_file_obj.id {
		println!(
			"SHOULD NOT HAPPEN. File id not matching {} {}",
			oa_file.id, asst_file_obj.id
		)
	}

	Ok((asst_file_obj.id.into(), true))
}

// endregion: --- Files
