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

fn get_directory_content_recursively(
    dir: &String,
    results: &mut Vec<CompareResult>,
) -> HashSet<String> {
    let mut filenames: HashSet<String> = HashSet::new();

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
                results.push(error);
            }
            Ok(entry) => {
                let f_name = entry
                    .path()
                    .strip_prefix(&dir)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                filenames.insert(f_name);
            }
        }
    }

    filenames
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

// TODO: How about returning errors instead of mutating it?
fn compare_file_contents(
    dir_a_content: &HashSet<String>,
    dir_b_content: &HashSet<String>,
    dir_a_path: &String,
    dir_b_path: &String,
    results: &mut Vec<CompareResult>,
) {
    let present_in_both = dir_a_content.intersection(&dir_b_content);
    for path in present_in_both {
        let path_a = Path::new(&dir_a_path).join(path);
        let path_b = Path::new(&dir_b_path).join(path);
        if path_a.is_file() && path_b.is_file() {
            let hash_a = match get_file_content_hash(path_a) {
                Err(why) => {
                    results.push(CompareResult::CouldNotCalculateHash(ErrorInfo {
                        path: dir_a_path.clone(),
                        message: why.to_string(),
                    }));
                    continue;
                }
                Ok(res) => res,
            };
            let hash_b = match get_file_content_hash(path_b) {
                Err(why) => {
                    results.push(CompareResult::CouldNotCalculateHash(ErrorInfo {
                        path: dir_b_path.clone(),
                        message: why.to_string(),
                    }));
                    continue;
                }
                Ok(res) => res,
            };
            if hash_a != hash_b {
                results.push(CompareResult::DifferingContent(EntryInfo {
                    path: path.clone(),
                }));
            }
        } else if !(path_a.is_dir() && path_b.is_dir()) {
            let entry_type = CompareResult::TypeMismatch(EntryTypeMismatch {
                path: path.clone(),
                type_in_dir_a: get_entry_type(&path_a),
                type_in_dir_b: get_entry_type(&path_b),
            });
            results.push(entry_type);
        }
    }
}

fn find_missing_entries(
    dir_a_content: &HashSet<String>,
    dir_b_content: &HashSet<String>,
    results: &mut Vec<CompareResult>,
) {
    let missing_in_dir_a =
        remove_subdirectories(dir_b_content.difference(&dir_a_content).into_iter())
            .map(|path| CompareResult::MissingInDirA(EntryInfo { path: path.clone() }));

    let missing_in_dir_b =
        remove_subdirectories(dir_a_content.difference(&dir_b_content).into_iter())
            .map(|path| CompareResult::MissingInDirB(EntryInfo { path: path.clone() }));

    let new_errors: Vec<CompareResult> = missing_in_dir_a.chain(missing_in_dir_b).collect();
    results.extend(new_errors);
}

#[tauri::command]
fn compare(path_a: String, path_b: String) -> Vec<CompareResult> {
    println!("received2");

    let mut results: Vec<CompareResult> = vec![];

    let dir_a_content = get_directory_content_recursively(&path_a, &mut results);
    let dir_b_content = get_directory_content_recursively(&path_b, &mut results);

    find_missing_entries(&dir_a_content, &dir_b_content, &mut results);

    compare_file_contents(
        &dir_a_content,
        &dir_b_content,
        &path_a,
        &path_b,
        &mut results,
    );

    results.into()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![compare])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]

mod tests {
    use super::*;

    fn call_structure_compare(path: &str) -> Vec<CompareResult> {
        // TODO: Rename "errors" to "result"
        let mut results: Vec<CompareResult> = vec![];
        find_missing_entries(
            &get_directory_content_recursively(
                &("./test/".to_string() + path + "/dirA"),
                &mut results,
            ),
            &get_directory_content_recursively(
                &("./test/".to_string() + path + "/dirB"),
                &mut results,
            ),
            &mut results,
        );
        results
    }
    fn call_content_compare(path: &str) -> Vec<CompareResult> {
        let path_a = String::from("./test/") + path + "/dirA";
        let path_b = String::from("./test/") + path + "/dirB";
        let mut results: Vec<CompareResult> = vec![];
        compare_file_contents(
            &get_directory_content_recursively(&path_a, &mut results),
            &get_directory_content_recursively(&path_b, &mut results),
            &path_a,
            &path_b,
            &mut results,
        );
        return results;
    }

    #[test]
    fn compare_invalid_directory() {
        assert_eq!(
            call_structure_compare("i_do_not_exist"),
                vec![
                  CompareResult::CouldNotReadDirectory(ErrorInfo {
                        path: String::from("./test/i_do_not_exist/dirA"),
                        message: String::from("IO error for operation on ./test/i_do_not_exist/dirA: No such file or directory (os error 2)")
                    }),
                    CompareResult::CouldNotReadDirectory(ErrorInfo {
                      path: String::from("./test/i_do_not_exist/dirB"),
                      message: String::from("IO error for operation on ./test/i_do_not_exist/dirB: No such file or directory (os error 2)")
                    })
                ]
        );
    }

    #[test]
    fn hash_invalid_file() {
        let mut results: Vec<CompareResult> = vec![];
        // Use /etc/sudoers to test a file we are not allowed to read
        compare_file_contents(
            &HashSet::from([String::from("/etc/sudoers")]),
            &HashSet::from([String::from("/etc/sudoers")]),
            &String::from("/etc/sudoers"),
            &String::from("/etc/sudoers"),
            &mut results,
        );
        assert_eq!(
            results,
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
