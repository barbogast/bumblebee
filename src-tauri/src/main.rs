#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use data_encoding::HEXUPPER;
use itertools::Itertools;
use ring::digest::{Context, Digest, SHA256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, PartialOrd, Clone, serde::Serialize)]
enum EntryType {
    Directory,
    File,
    // Link, // TODO
    Unknown,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, serde::Serialize)]
struct EntryTypeMismatch {
    path: String,
    type_in_dir_a: EntryType,
    type_in_dir_b: EntryType,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, serde::Serialize)]
struct EntryInfo {
    path: String,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, serde::Serialize)]
struct ErrorInfo {
    path: String,
    message: String,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, serde::Serialize)]
#[serde(tag = "type")]
enum CompareResult {
    CouldNotReadDirectory(ErrorInfo),
    CouldNotCalculateHash(ErrorInfo),
    MissingInDirA(EntryInfo),
    MissingInDirB(EntryInfo),
    DifferingContent(EntryInfo),
    TypeMismatch(EntryTypeMismatch),
}

fn get_directory_content_recursively(dir: &String) -> (HashSet<String>, Vec<CompareResult>) {
    let mut filenames: HashSet<String> = HashSet::new();
    let mut errors: Vec<CompareResult> = Vec::new();

    for result in WalkDir::new(&dir).into_iter() {
        match result {
            Err(why) => {
                let error = CompareResult::CouldNotReadDirectory(ErrorInfo {
                    path: why
                        .path()
                        .unwrap_or(Path::new(""))
                        .to_string_lossy()
                        .to_string(),
                    message: why.to_string(),
                });
                errors.push(error);
            }
            Ok(entry) => {
                let f_name = entry
                    .path()
                    .strip_prefix(&dir)
                    // This should never panic as the path should always start with the base directory
                    .expect("Path doesn't sart with base directory")
                    .to_string_lossy()
                    .to_string();
                filenames.insert(f_name);
            }
        }
    }

    (filenames, errors)
}

// When handling missing directories / files the initial list contains missing directories and each missing file.
// In this case we only need to know that the directory is missing, so let's filter out the contents.
fn remove_subdirectories<'a, I>(paths: I) -> impl Iterator<Item = &'a String>
where
    I: Iterator<Item = &'a String>,
{
    paths
        // Sort entries aphabetially, then only keep an entry if its beginning doesn't match
        // the previous one.
        // This relies on /my_dir appearing before /my_dir/file, in which case the latter would be dropped
        .sorted()
        .coalesce(|a, b| if b.starts_with(a) { Ok(a) } else { Err((a, b)) })
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, io::Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn get_file_content_hash<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader)?;
    Ok(HEXUPPER.encode(digest.as_ref()))
}

fn get_entry_type(path: &Path) -> EntryType {
    if path.is_dir() {
        EntryType::Directory
    } else if path.is_file() {
        EntryType::File
    }
    // TODO: is_symlink() is only allowed in nightly...
    // else if path.is_symlink() { EntryType::Link }
    else {
        EntryType::Unknown
    }
}

fn compare_entry(
    dir_a_path: &String,
    dir_b_path: &String,
    sub_path: String,
) -> Result<(), CompareResult> {
    let path_a = Path::new(&dir_a_path).join(&sub_path);
    let path_b = Path::new(&dir_b_path).join(&sub_path);
    if path_a.is_file() && path_b.is_file() {
        let hash_a = (get_file_content_hash(&path_a).map_err(|why| {
            CompareResult::CouldNotCalculateHash(ErrorInfo {
                path: path_a.to_string_lossy().to_string(),
                message: why.to_string(),
            })
        }))?;
        let hash_b = get_file_content_hash(&path_b).map_err(|why| {
            CompareResult::CouldNotCalculateHash(ErrorInfo {
                path: path_b.to_string_lossy().to_string(),
                message: why.to_string(),
            })
        })?;
        if hash_a != hash_b {
            return Err(CompareResult::DifferingContent(EntryInfo {
                path: sub_path.clone(),
            }));
        }
    } else if !(path_a.is_dir() && path_b.is_dir()) {
        return Err(CompareResult::TypeMismatch(EntryTypeMismatch {
            path: sub_path.clone(),
            type_in_dir_a: get_entry_type(&path_a),
            type_in_dir_b: get_entry_type(&path_b),
        }));
    }
    Ok(())
}

fn compare_directory_contents<'a>(
    dir_a_content: &'a HashSet<String>,
    dir_b_content: &'a HashSet<String>,
    dir_a_path: &'a String,
    dir_b_path: &'a String,
) -> impl Iterator<Item = CompareResult> + 'a {
    let present_in_both = dir_a_content.intersection(&dir_b_content);
    present_in_both.filter_map(|path| compare_entry(dir_a_path, dir_b_path, path.to_string()).err())
}

fn find_missing_entries<'a>(
    dir_a_content: &'a HashSet<String>,
    dir_b_content: &'a HashSet<String>,
) -> Box<dyn Iterator<Item = CompareResult> + 'a> {
    let missing_in_dir_a =
        remove_subdirectories(dir_b_content.difference(&dir_a_content).into_iter())
            .map(|path| CompareResult::MissingInDirA(EntryInfo { path: path.clone() }));

    let missing_in_dir_b =
        remove_subdirectories(dir_a_content.difference(&dir_b_content).into_iter())
            .map(|path| CompareResult::MissingInDirB(EntryInfo { path: path.clone() }));

    Box::new(missing_in_dir_a.chain(missing_in_dir_b))
}

#[tauri::command]
fn compare(path_a: String, path_b: String) -> Vec<CompareResult> {
    println!("received2");

    let (dir_a_content, dir_a_errors) = get_directory_content_recursively(&path_a);
    let (dir_b_content, dir_b_errors) = get_directory_content_recursively(&path_b);

    vec![]
        .into_iter()
        .chain(dir_a_errors)
        .chain(dir_b_errors)
        .chain(find_missing_entries(&dir_a_content, &dir_b_content))
        .chain(compare_directory_contents(
            &dir_a_content,
            &dir_b_content,
            &path_a,
            &path_b,
        ))
        .collect::<Vec<CompareResult>>()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![compare, copy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]

mod tests {
    use super::*;

    fn call_structure_compare(path: &str) -> Vec<CompareResult> {
        let (dir_content_a, dir_a_errors) =
            get_directory_content_recursively(&("./test/".to_string() + path + "/dirA"));
        let (dir_content_b, dir_b_errors) =
            get_directory_content_recursively(&("./test/".to_string() + path + "/dirB"));
        assert_eq!(dir_a_errors, vec![]);
        assert_eq!(dir_b_errors, vec![]);
        find_missing_entries(&dir_content_a, &dir_content_b).collect()
    }
    fn call_content_compare(path: &str) -> Vec<CompareResult> {
        let path_a = "./test/".to_string() + path + "/dirA";
        let path_b = "./test/".to_string() + path + "/dirB";
        let (dir_content_a, dir_a_errors) = get_directory_content_recursively(&path_a);
        let (dir_content_b, dir_b_errors) = get_directory_content_recursively(&path_b);
        assert_eq!(dir_a_errors, vec![]);
        assert_eq!(dir_b_errors, vec![]);
        compare_directory_contents(&dir_content_a, &dir_content_b, &path_a, &path_b).collect()
    }

    #[test]
    fn read_invalid_directory() {
        assert_eq!(
            get_directory_content_recursively(&("i_do_not_exist".to_string())),
                (HashSet::new(), vec![
                  CompareResult::CouldNotReadDirectory(ErrorInfo {
                        path: String::from("i_do_not_exist"),
                        message: String::from("IO error for operation on i_do_not_exist: No such file or directory (os error 2)")
                    }),
                ])
        );
    }

    #[test]
    fn hash_invalid_file() {
        // Use /etc/sudoers to test a file we are not allowed to read
        let dir = String::from("/etc/sudoers");
        let dir_content = &HashSet::from([dir.clone()]);
        let results = compare_directory_contents(dir_content, dir_content, &dir, &dir);
        assert_eq!(
            results.collect::<Vec<CompareResult>>(),
            vec![CompareResult::CouldNotCalculateHash(ErrorInfo {
                path: String::from("/etc/sudoers"),
                message: String::from("Permission denied (os error 13)")
            })]
        );
    }

    #[test]
    fn t_01_test_files_match() {
        assert_eq!(call_structure_compare("01_test_files_match"), vec![]);
    }

    #[test]
    fn t_02_dir_a_lacks_file() {
        assert_eq!(
            call_structure_compare("02_dirA_lacks_file"),
            vec![CompareResult::MissingInDirA(EntryInfo {
                path: String::from("file2.txt")
            })]
        );
    }

    #[test]
    fn t_03_dir_b_lacks_file() {
        assert_eq!(
            call_structure_compare("03_dirB_lacks_file"),
            vec![CompareResult::MissingInDirB(EntryInfo {
                path: String::from("file1.txt")
            })]
        );
    }

    #[test]
    fn t_04_dir_a_lacks_sub_directory() {
        assert_eq!(
            call_structure_compare("04_dirA_lacks_sub_directory"),
            vec![CompareResult::MissingInDirA(EntryInfo {
                path: String::from("subdir2")
            })]
        );
    }

    #[test]
    fn t_05_dir_a_lacks_file_in_sub_directory() {
        assert_eq!(
            call_structure_compare("05_dirA_lacks_file_in_sub_directory"),
            vec![CompareResult::MissingInDirA(EntryInfo {
                path: String::from("subdir2/file2.txt")
            })]
        );
    }

    #[test]
    fn t_06_different_text_content() {
        assert_eq!(
            call_content_compare("06_different_text_content"),
            vec![CompareResult::DifferingContent(EntryInfo {
                path: String::from("file1.txt")
            })]
        );
    }

    #[test]
    fn t_07_different_binary_content() {
        assert_eq!(
            call_content_compare("07_different_binary_content"),
            vec![CompareResult::DifferingContent(EntryInfo {
                path: String::from("file1.jpeg")
            })]
        );
    }

    #[test]
    fn t_08_type_mismatch() {
        assert_eq!(
            call_content_compare("08_type_mismatch"),
            vec![CompareResult::TypeMismatch(EntryTypeMismatch {
                path: String::from("file1.txt"),
                type_in_dir_a: EntryType::File,
                type_in_dir_b: EntryType::Directory
            })],
        );
    }
}
