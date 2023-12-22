use crate::ais::msg::{get_text_content, user_msg};
use crate::ais::OaClient;
use crate::utils::cli::{
	ico_check, ico_deleted_ok, ico_err, ico_uploaded, ico_uploading,
};
use crate::utils::files::XFile;
use crate::Result;
use async_openai::types::{
	AssistantObject, AssistantToolsRetrieval, CreateAssistantFileRequest,
	CreateAssistantRequest, CreateFileRequest, CreateRunRequest,
	CreateThreadRequest, ModifyAssistantRequest, RunStatus, ThreadObject,
};
use console::Term;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
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

#[derive(Debug, From, Deref, Display)]
pub struct AsstId(String);

#[derive(Debug, From, Deref, Display, Serialize, Deserialize)]
pub struct ThreadId(String);

#[derive(Debug, From, Deref, Display)]
pub struct FileId(String);

// endregion: --- Types

// region:    --- Asst CRUD

pub async fn create(oac: &OaClient, config: CreateConfig) -> Result<AsstId> {
	let oa_assts = oac.assistants();

	let asst_obj = oa_assts
		.create(CreateAssistantRequest {
			model: config.model,
			name: Some(config.name),
			tools: Some(vec![AssistantToolsRetrieval::default().into()]),
			..Default::default()
		})
		.await?;

	Ok(asst_obj.id.into())
}

pub async fn load_or_create(
	oac: &OaClient,
	config: CreateConfig,
	recreate: bool,
) -> Result<AsstId> {
	let asst_obj = first_by_name(oac, &config.name).await?;
	let mut asst_id = asst_obj.map(|o| AsstId::from(o.id));

	// -- Delete asst if recreate true and asst_id
	if let (true, Some(asst_id_ref)) = (recreate, asst_id.as_ref()) {
		delete(oac, asst_id_ref).await?;
		asst_id.take();
		println!("{} Assistant {} deleted", ico_deleted_ok(), config.name);
	}

	// -- Create if needed
	if let Some(asst_id) = asst_id {
		println!("{} Assistant {} loaded", ico_check(), config.name);
		Ok(asst_id)
	} else {
		let asst_name = config.name.clone();
		let asst_id = create(oac, config).await?;
		println!("{} Assistant {} created", ico_check(), asst_name);
		Ok(asst_id)
	}
}

pub async fn first_by_name(
	oac: &OaClient,
	name: &str,
) -> Result<Option<AssistantObject>> {
	let oa_assts = oac.assistants();

	let assts = oa_assts.list(DEFAULT_QUERY).await?.data;

	let asst_obj = assts
		.into_iter()
		.find(|a| a.name.as_ref().map(|n| n == name).unwrap_or(false));

	Ok(asst_obj)
}

pub async fn upload_instructions(
	oac: &OaClient,
	asst_id: &AsstId,
	inst_content: String,
) -> Result<()> {
	let oa_assts = oac.assistants();
	let modif = ModifyAssistantRequest {
		instructions: Some(inst_content),
		..Default::default()
	};
	oa_assts.update(asst_id, modif).await?;

	Ok(())
}

pub async fn delete(oac: &OaClient, asst_id: &AsstId) -> Result<()> {
	let oa_assts = oac.assistants();
	let oa_files = oac.files();

	// -- First delete the files associated to this assistant.
	for file_id in get_files_hashmap(oac, asst_id).await?.into_values() {
		let del_res = oa_files.delete(&file_id).await;
		// NOTE: Might be already deleted, that's ok for now.
		if del_res.is_ok() {
			println!("{} file deleted - {file_id}", ico_deleted_ok());
		}
	}

	// Note: No need to delete assistant files since we delete the assistant.

	// -- Delete assistant
	oa_assts.delete(asst_id).await?;

	Ok(())
}

// endregion: --- Asst CRUD

// region:    --- Thread

pub async fn create_thread(oac: &OaClient) -> Result<ThreadId> {
	let oa_threads = oac.threads();

	let res = oa_threads
		.create(CreateThreadRequest {
			..Default::default()
		})
		.await?;

	Ok(res.id.into())
}

pub async fn get_thread(
	oac: &OaClient,
	thread_id: &ThreadId,
) -> Result<ThreadObject> {
	let oa_threads = oac.threads();

	let thread_obj = oa_threads.retrieve(thread_id).await?;

	Ok(thread_obj)
}

pub async fn run_thread_msg(
	oac: &OaClient,
	asst_id: &AsstId,
	thread_id: &ThreadId,
	msg: &str,
) -> Result<String> {
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
				return get_first_thread_msg_content(oac, thread_id).await;
			}
			RunStatus::Queued | RunStatus::InProgress => (),
			other => {
				term.write_str("\n")?;
				return Err(format!("ERROR WHILE RUN: {:?}", other).into());
			}
		}

		sleep(Duration::from_millis(POLLING_DURATION_MS)).await;
	}
}

pub async fn get_first_thread_msg_content(
	oac: &OaClient,
	thread_id: &ThreadId,
) -> Result<String> {
	static QUERY: [(&str, &str); 1] = [("limit", "1")];

	let messages = oac.threads().messages(thread_id).list(&QUERY).await?;
	let msg = messages
		.data
		.into_iter()
		.next()
		.ok_or_else(|| "No message found".to_string())?;

	let text = get_text_content(msg)?;

	Ok(text)
}

// endregion: --- Thread

// region:    --- Files

/// Returns the file id by file name hashmap.
pub async fn get_files_hashmap(
	oac: &OaClient,
	asst_id: &AsstId,
) -> Result<HashMap<String, FileId>> {
	// -- Get all asst files (files do not have .name)
	let oas_assts = oac.assistants();
	let oa_asst_files = oas_assts.files(asst_id);
	let asst_files = oa_asst_files.list(DEFAULT_QUERY).await?.data;
	let asst_file_ids: HashSet<String> =
		asst_files.into_iter().map(|f| f.id).collect();

	// -- Get all files for org (those files have .filename)
	let oa_files = oac.files();
	let org_files = oa_files.list().await?.data;

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
	oac: &OaClient,
	asst_id: &AsstId,
	file: &Path,
	force: bool,
) -> Result<(FileId, bool)> {
	let file_name = file.x_file_name();
	let mut file_id_by_name = get_files_hashmap(oac, asst_id).await?;

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
			println!(
				"{} Can't delete file '{}'\n    cause: {}",
				ico_err(),
				file.to_string_lossy(),
				err
			);
		}

		// -- Delete the asst_file association
		let oa_assts = oac.assistants();
		let oa_assts_files = oa_assts.files(asst_id);
		if let Err(err) = oa_assts_files.delete(&file_id).await {
			println!(
				"{} Can't remove assistant file '{}'\n    cause: {}",
				ico_err(),
				file.x_file_name(),
				err
			);
		}
	}

	// -- Upload and attach the file.
	let term = Term::stdout();

	// Print uploading.
	term.write_line(&format!(
		"{} Uploading file '{}'",
		ico_uploading(),
		file.x_file_name()
	))?;

	// Upload file.
	let oa_files = oac.files();
	let oa_file = oa_files
		.create(CreateFileRequest {
			file: file.into(),
			purpose: "assistants".into(),
		})
		.await?;

	// Update print.
	term.clear_last_lines(1)?;
	term.write_line(&format!(
		"{} Uploaded file '{}'",
		ico_uploaded(),
		file.x_file_name()
	))?;

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
