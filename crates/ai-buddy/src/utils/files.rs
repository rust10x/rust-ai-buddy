use crate::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufRead as _, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// region:    --- File Bundler

pub fn bundle_to_file(files: Vec<PathBuf>, dst_file: &Path) -> Result<()> {
	let mut writer = BufWriter::new(File::create(dst_file)?);

	for file in files {
		if !file.is_file() {
			return Err(Error::FileCannotBundleNoneFile(
				file.to_string_lossy().to_string(),
			));
		}
		let reader = get_reader(&file)?;

		writeln!(writer, "\n// ==== file path: {}\n", file.to_string_lossy())?;

		for line in reader.lines() {
			let line = line?;
			writeln!(writer, "{}", line)?;
		}
		writeln!(writer, "\n\n")?;
	}
	writer.flush()?;

	Ok(())
}

// endregion: --- File Bundler

// region:    --- File Parser/Writer

pub fn load_from_toml<T>(file: impl AsRef<Path>) -> Result<T>
where
	T: serde::de::DeserializeOwned,
{
	let content = read_to_string(file.as_ref())?;

	Ok(toml::from_str(&content)?)
}

pub fn load_from_json<T>(file: impl AsRef<Path>) -> Result<T>
where
	T: serde::de::DeserializeOwned,
{
	let val = serde_json::from_reader(get_reader(file.as_ref())?)?;
	Ok(val)
}

pub fn save_to_json<T>(file: impl AsRef<Path>, data: &T) -> Result<()>
where
	T: serde::Serialize,
{
	let file = file.as_ref();

	let file = File::create(file).map_err(|e| Error::FileCannotCreate {
		file: file.to_string_lossy().to_ascii_lowercase(),
		cause: e,
	})?;

	serde_json::to_writer_pretty(file, data)?;

	Ok(())
}

// endregion: --- File Parser/Writer

// region:    --- Dir Utils

/// Returns true if one or more dir was created.
pub fn ensure_dir(dir: &Path) -> Result<bool> {
	if dir.is_dir() {
		Ok(false)
	} else {
		fs::create_dir_all(dir)?;
		Ok(true)
	}
}

pub fn list_files(
	dir: &Path,
	include_globs: Option<&[&str]>,
	exclude_globs: Option<&[&str]>,
) -> Result<Vec<PathBuf>> {
	let base_dir_exclude = base_dir_exclude_globs()?;

	// -- Determine recursive depth
	let depth = include_globs
		.map(|globs| globs.iter().any(|&g| g.contains("**")))
		.map(|v| if v { 100 } else { 1 })
		.unwrap_or(1);

	// -- Prep globs
	let include_globs = include_globs.map(get_glob_set).transpose()?;
	let exclude_globs = exclude_globs.map(get_glob_set).transpose()?;

	// -- Build file iterator
	let walk_dir_it = WalkDir::new(dir)
		.max_depth(depth)
		.into_iter()
		.filter_entry(|e|
			// if dir, check the dir exclude
			if e.file_type().is_dir() {
				!base_dir_exclude.is_match(e.path())
			}
			// else file, we apply the globs
			else {
				// first, evaluate the exclude
				if let Some(exclude_globs) = exclude_globs.as_ref() {
					if exclude_globs.is_match(e.path()) {
						return false;
					}
				}
				// otherwise, evaluate the include
				match include_globs.as_ref() {
					Some(globs) => globs.is_match(e.path()),
					None => true,
    		}
			}
		)
		.filter_map(|e| e.ok().filter(|e| e.file_type().is_file()));

	let paths = walk_dir_it.map(|e| e.into_path());

	Ok(paths.collect())
}

fn base_dir_exclude_globs() -> Result<GlobSet> {
	get_glob_set(&["**/.git", "**/target"])
}

pub fn get_glob_set(globs: &[&str]) -> Result<GlobSet> {
	let mut builder = GlobSetBuilder::new();
	for glob in globs {
		builder.add(Glob::new(glob)?);
	}
	Ok(builder.build()?)
}

// endregion: --- Dir Utils

// region:    --- File Utils

pub fn read_to_string(file: &Path) -> Result<String> {
	if !file.is_file() {
		return Err(Error::FileNotFound(file.to_string_lossy().to_string()));
	}
	let content = fs::read_to_string(file)?;

	Ok(content)
}

fn get_reader(file: &Path) -> Result<BufReader<File>> {
	let Ok(file) = File::open(file) else {
		return Err(Error::FileNotFound(file.display().to_string()));
	};

	Ok(BufReader::new(file))
}

// endregion: --- File Utils

// region:    --- XFile

/// Trait that has methods that returns
/// the `&str` when ok, and when none or err, returns ""
pub trait XFile {
	fn x_file_name(&self) -> &str;
	fn x_extension(&self) -> &str;
}

impl XFile for Path {
	fn x_file_name(&self) -> &str {
		self.file_name().and_then(OsStr::to_str).unwrap_or("")
	}

	fn x_extension(&self) -> &str {
		self.extension().and_then(OsStr::to_str).unwrap_or("")
	}
}

// endregion: --- XFile
