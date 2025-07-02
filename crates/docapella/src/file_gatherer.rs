use libdoctave::{InputContent, InputFile};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub(crate) fn gather_files(
    working_dir: &Path,
) -> std::result::Result<Vec<InputFile>, std::io::Error> {
    let mut files = Vec::new();

    for entry in fs::read_dir(working_dir)? {
        let path = entry?.path();

        if path.is_dir() {
            files.extend(gather_files(&path)?);
        } else {
            files.push(InputFile {
                path: path.strip_prefix(working_dir).expect("Found file was not in working dir").to_path_buf(),
                content: match std::fs::read_to_string(&path) {
                    Ok(s) => Ok(InputContent::Text(s)),
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::InvalidData {
                            match std::fs::metadata(&path)
                                .and_then(|meta| meta.modified())
                                .and_then(|system_time| {
                                    system_time.duration_since(UNIX_EPOCH).map_err(|e| {
                                        std::io::Error::new(
                                            std::io::ErrorKind::InvalidData,
                                            e.to_string(),
                                        )
                                    })
                                }) {
                                Ok(modified_time) => {
                                    Ok(InputContent::Binary(modified_time.as_millis().to_string()))
                                }
                                Err(e) => Err(e),
                            }
                        } else {
                            Err(e)
                        }
                    }
                }?,
            });
        }
    }

    return Ok(files);
}
