//! Atomic export shared by both front ends. Source aliases are never replaced.

use std::{fs, io, os::unix::fs::MetadataExt, path::Path};

use crate::{Result, bmp, mono::MonoImage};

pub fn save(image: &MonoImage, source: &Path, output: &Path, overwrite: bool) -> Result<()> {
    let source_metadata = fs::metadata(source)?;
    match fs::metadata(output) {
        Ok(destination) => {
            if source_metadata.dev() == destination.dev()
                && source_metadata.ino() == destination.ino()
            {
                return Err(
                    "output is the source image (or an alias); choose a different path".into(),
                );
            }
            if !overwrite {
                return Err(
                    "output already exists; confirm replacement (--force in the CLI)".into(),
                );
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    bmp::write(image, temporary.as_file_mut())?;
    temporary.as_file().sync_all()?;
    if overwrite {
        temporary.persist(output)?;
    } else {
        temporary.persist_noclobber(output)?;
    }
    Ok(())
}
