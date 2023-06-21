use super::errors::LayoutError;
use std::{fs, path::Path};

pub type FileInfos = (String, String);
pub type DirectoryName = String;

pub fn get_directory_layout(
    current_path: &str,
) -> Result<(DirectoryName, Vec<FileInfos>), LayoutError> {
    let current_path = Path::new(current_path);

    let parent_directory = current_path
        .parent()
        .ok_or_else(|| LayoutError::Other("Cannot get current directory".to_string()))?;

    let dir_files_names = fs::read_dir(parent_directory)
        .map_err(|e| LayoutError::CannotReadDirectory {
            error: e.to_string(),
            directory: parent_directory.to_string_lossy().to_string(),
        })?
        .filter_map(|entry| {
            let file = match entry {
                Ok(x) => match entry_to_file_infos(x) {
                    Ok(Some(x)) => Some(Ok(x)),
                    Ok(None) => None,
                    Err(x) => Some(Err(x)),
                },
                Err(e) => Some(Err(LayoutError::CannotGetDirectoryEntry {
                    error: e.to_string(),
                })),
            };

            file
        })
        .collect::<Result<Vec<FileInfos>, LayoutError>>()?;

    let parent_directory_name = parent_directory
        .file_name()
        .ok_or_else(|| LayoutError::Other("Cannot get current directory".to_string()))?
        .to_string_lossy()
        .to_string();

    match dir_files_names.len() {
        0 => Err(LayoutError::NoFilesFound),
        _ => Ok((parent_directory_name, dir_files_names)),
    }
}

fn entry_to_file_infos(entry: fs::DirEntry) -> Result<Option<FileInfos>, LayoutError> {
    let entry_name = entry.file_name().to_string_lossy().to_string();
    let entry_no_ext = Path::new(&entry_name)
        .file_stem()
        .ok_or_else(|| LayoutError::Other("File doesn't have a name".to_string()))
        .map(|stem| stem.to_string_lossy().to_string())?;
    let entry_fully_qualified = entry.path().to_string_lossy().to_string();

    if !entry_name.ends_with(".rs") {
        return Ok(None);
    }

    if entry_no_ext == "mod" {
        return Ok(None);
    }

    return Ok(Some((entry_no_ext, entry_fully_qualified)));
}
