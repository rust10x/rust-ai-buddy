use crate::Result;
use simple_fs::{get_buf_reader, SFile, SPath};
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufWriter};

pub fn bundle_to_file(files: Vec<SFile>, dst_file: &SPath) -> Result<()> {
	let mut writer = BufWriter::new(File::create(dst_file)?);

	for file in files {
		let reader = get_buf_reader(&file)?;

		writeln!(writer, "\n// ==== file path: {file}\n")?;

		for line in reader.lines() {
			let line = line?;
			writeln!(writer, "{}", line)?;
		}
		writeln!(writer, "\n\n")?;
	}
	writer.flush()?;

	Ok(())
}
