#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use data_encoding::HEXUPPER;
use fs_extra;
use itertools::Itertools;
use ring::digest::{Context, Digest, SHA256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
enum EntryType {
    Directory,
    File,
    // Link, // TODO
    Unknown,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
struct EntryTypeMismatch {
    path: String,
    type_in_dir_a: EntryType,
    type_in_dir_b: EntryType,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
struct EntryInfo {
    path: String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
struct ErrorInfo {
    path: String,
    message: String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum CompareResult {
    CouldNotReadDirectory(ErrorInfo),
    CouldNotCalculateHash(ErrorInfo),
    MissingInDirA(EntryInfo),
    MissingInDirB(EntryInfo),
    DifferingContent(EntryInfo),
    TypeMismatch(EntryTypeMismatch),
}

fn get_directory_content_recursively(dir: &str) -> (HashSet<String>, Vec<CompareResult>) {
    let mut filenames: HashSet<String> = HashSet::new();
    let mut errors: Vec<CompareResult> = Vec::new();

    for result in WalkDir::new(&dir) {
        match result {
            Err(why) => {
                let error = CompareResult::CouldNotReadDirectory(ErrorInfo {
                    path: why
                        .path()
                        .unwrap_or_else(|| Path::new(""))
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
                    .expect("Path doesn't start with base directory")
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
    } else {
        EntryType::Unknown
    }
    // TODO: is_symlink() is only allowed in nightly...
    // else if path.is_symlink() { EntryType::Link }
}

fn compare_entry(
    dir_a_path: &str,
    dir_b_path: &str,
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
                path: sub_path,
            }));
        }
    } else if !(path_a.is_dir() && path_b.is_dir()) {
        return Err(CompareResult::TypeMismatch(EntryTypeMismatch {
            path: sub_path,
            type_in_dir_a: get_entry_type(&path_a),
            type_in_dir_b: get_entry_type(&path_b),
        }));
    }
    Ok(())
}

fn compare_directory_contents<'a>(
    dir_a_content: &'a HashSet<String>,
    dir_b_content: &'a HashSet<String>,
    dir_a_path: &'a str,
    dir_b_path: &'a str,
) -> impl Iterator<Item = CompareResult> + 'a {
    let present_in_both = dir_a_content.intersection(dir_b_content);
    present_in_both.filter_map(|path| compare_entry(dir_a_path, dir_b_path, path.to_string()).err())
}

fn find_missing_entries<'a>(
    dir_a_content: &'a HashSet<String>,
    dir_b_content: &'a HashSet<String>,
) -> Box<dyn Iterator<Item = CompareResult> + 'a> {
    let missing_in_dir_a = remove_subdirectories(dir_b_content.difference(dir_a_content))
        .map(|path| CompareResult::MissingInDirA(EntryInfo { path: path.clone() }));

    let missing_in_dir_b = remove_subdirectories(dir_a_content.difference(dir_b_content))
        .map(|path| CompareResult::MissingInDirB(EntryInfo { path: path.clone() }));

    Box::new(missing_in_dir_a.chain(missing_in_dir_b))
}

#[tauri::command]
fn compare(path_a: String, path_b: String) -> Vec<CompareResult> {
    println!("received2");

    let (dir_a_content, dir_a_errors) = get_directory_content_recursively(&path_a);
    let (dir_b_content, dir_b_errors) = get_directory_content_recursively(&path_b);

    let mut res = vec![]
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
        .collect::<Vec<CompareResult>>();

    // Sort so the order of the reuslts doesn't change between runs
    // This is important for the tests but probably also reasonable for the user
    res.sort();

    res
}

#[tauri::command]
fn copy(source_path: String, target_path: String, sub_paths: Vec<String>) -> Vec<ErrorInfo> {
    dbg!(&source_path, &target_path, &sub_paths);
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    sub_paths
        .into_iter()
        .filter_map(|path| {
            fs_extra::copy_items(
                &[Path::new(&source_path).join(&path)],
                &Path::new(&target_path),
                &options,
            )
            .map_err(|error| ErrorInfo {
                message: error.to_string(),
                path,
            })
            .err()
        })
        .collect()
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
    use tempfile::tempdir;

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

    /// Creates a temporary directory under /tmp and copies the given `path` to this directory
    fn create_test_directory(path: &str) -> fs_extra::error::Result<tempfile::TempDir> {
        let dir = tempdir()?;
        dbg!("Copy {} to {}", &path, dir.path());
        fs_extra::copy_items(&[path], &dir, &fs_extra::dir::CopyOptions::new())?;
        Ok(dir)
    }

    #[test]
    fn read_invalid_directory() {
        assert_eq!(
            get_directory_content_recursively(&("i_do_not_exist".to_string())),
            (
                HashSet::new(),
                vec![CompareResult::CouldNotReadDirectory(ErrorInfo {
                    path: String::from("i_do_not_exist"),
                    message: String::from(
                        "IO error for operation on i_do_not_exist: No such file or directory (os error 2)"
                    )
                }),]
            )
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

    // To test copying files we:
    //   1. Copy the folder of "00_all_cases" to a new folder in /tmp
    //   2. Make sure that dirA and dirB contain the expected differences
    //   3. Run copy() for one of the differences
    //   4. Run the comparison again to assert that the expected difference disappeared
    // Note that 2, 3 and 4 are all executed on the copied directoy in /tmp
    #[test]
    fn test_copy_one_file() -> Result<(), fs_extra::error::Error> {
        let dir = create_test_directory("test/03_dirB_lacks_file")?;
        let base_path = dir.path().join("03_dirB_lacks_file");
        let path_a = base_path.join("dirA").to_string_lossy().to_string();
        let path_b = base_path.join("dirB").to_string_lossy().to_string();

        assert_eq!(
            compare(path_a.clone(), path_b.clone()),
            vec![CompareResult::MissingInDirB(EntryInfo {
                path: "file1.txt".to_string(),
            }),]
        );

        let errors = copy(
            path_a.clone(),
            path_b.clone(),
            vec!["file1.txt".to_string()],
        );

        let expected_errors: Vec<ErrorInfo> = Vec::new();
        assert_eq!(errors, expected_errors);

        assert_eq!(compare(path_a, path_b), vec![]);

        Ok(())
    }

    #[test]
    fn test_copy_multiple_files() -> Result<(), fs_extra::error::Error> {
        let dir = create_test_directory("test/09_3_wrong_files")?;
        let base_path = dir.path().join("09_3_wrong_files");
        let path_a = base_path.join("dirA").to_string_lossy().to_string();
        let path_b = base_path.join("dirB").to_string_lossy().to_string();

        assert_eq!(
            compare(path_a.clone(), path_b.clone()),
            vec![
                CompareResult::MissingInDirB(EntryInfo {
                    path: "file_only_in_a.txt".to_string(),
                }),
                CompareResult::DifferingContent(EntryInfo {
                    path: "differing_content.txt".to_string(),
                }),
                CompareResult::DifferingContent(EntryInfo {
                    path: "differing_content2.txt".to_string(),
                })
            ]
        );

        // Let's copy file_only_in_a.txt and differing_content.txt but not differing_content2.txt
        let errors = copy(
            path_a.clone(),
            path_b.clone(),
            vec![
                "file_only_in_a.txt".to_string(),
                "differing_content.txt".to_string(),
            ],
        );

        let expected_errors: Vec<ErrorInfo> = Vec::new();
        assert_eq!(errors, expected_errors);

        assert_eq!(
            compare(path_a, path_b),
            vec![CompareResult::DifferingContent(EntryInfo {
                path: "differing_content2.txt".to_string(),
            })]
        );

        Ok(())
    }

    #[test]
    fn test_copy_directory() -> Result<(), fs_extra::error::Error> {
        let dir = create_test_directory("test/04_dirA_lacks_sub_directory")?;
        let base_path = dir.path().join("04_dirA_lacks_sub_directory");
        let path_a = base_path.join("dirA").to_string_lossy().to_string();
        let path_b = base_path.join("dirB").to_string_lossy().to_string();

        assert_eq!(
            compare(path_a.clone(), path_b.clone()),
            vec![CompareResult::MissingInDirA(EntryInfo {
                path: String::from("subdir2")
            })]
        );

        let errors = copy(path_b.clone(), path_a.clone(), vec!["subdir2".to_string()]);

        let expected_errors: Vec<ErrorInfo> = Vec::new();
        assert_eq!(errors, expected_errors);

        assert_eq!(compare(path_a, path_b), vec![]);

        Ok(())
    }
}
