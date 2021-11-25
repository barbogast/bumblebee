#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use data_encoding::HEXUPPER;
use itertools::Itertools;
use ring::digest::{Context, Digest, SHA256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use walkdir::WalkDir;

#[derive(Debug, PartialEq, PartialOrd)]
struct StructureCompareResult {
    missing_in_dir_a: Vec<String>,
    missing_in_dir_b: Vec<String>,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct ContentCompareResult {
    differing_content: Vec<String>,
    file_and_directory: Vec<String>,
}

fn get_directory_content_recursively(dir: &String) -> HashSet<String> {
    let mut filenames: HashSet<String> = HashSet::new();

    for result in WalkDir::new(&dir).into_iter() {
        match result {
            Err(why) => println!("! {:?}", why),
            Ok(entry) => {
                let f_name = entry.path().strip_prefix(&dir).unwrap().to_string_lossy();
                filenames.insert(f_name.to_string());
            }
        }
    }

    filenames
}

// When handling missing directories / files the initial list contains missing directories and each missing file.
// In this case we only need to know that the directory is missing, so let's filter out the contents.
fn remove_subdirectories(paths: &Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .sorted()
        .coalesce(|a, b| if b.starts_with(a) { Ok(a) } else { Err((a, b)) })
        .cloned()
        .collect()
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, std::io::Error> {
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

fn get_file_content_hash<P: AsRef<std::path::Path>>(path: P) -> Result<String, std::io::Error> {
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader)?;
    Ok(HEXUPPER.encode(digest.as_ref()))
}

fn compare_file_contents(
    dir_a_content: &HashSet<String>,
    dir_b_content: &HashSet<String>,
    dir_a_path: &String,
    dir_b_path: &String,
) -> ContentCompareResult {
    let mut differing_content: Vec<String> = Vec::new();
    let mut file_and_directory: Vec<String> = Vec::new();

    let present_in_both = dir_a_content.intersection(&dir_b_content);
    for path in present_in_both {
        let path_a = std::path::Path::new(&dir_a_path).join(path);
        let path_b = std::path::Path::new(&dir_b_path).join(path);
        if path_a.is_file() && path_b.is_file() {
            let hash_a = match get_file_content_hash(path_a) {
                Err(why) => {
                    println!("{:?}", why); // TODO: process error
                    continue;
                }
                Ok(res) => res,
            };
            let hash_b = match get_file_content_hash(path_b) {
                Err(why) => {
                    println!("{:?}", why); // TODO: process error
                    continue;
                }
                Ok(res) => res,
            };
            if hash_a != hash_b {
                differing_content.push(path.clone());
            }
        } else if !(path_a.is_dir() && path_b.is_dir()) {
            file_and_directory.push(path.clone());
        }
    }

    ContentCompareResult {
        differing_content,
        file_and_directory,
    }
}

fn analyze(
    dir_a_content: &HashSet<String>,
    dir_b_content: &HashSet<String>,
) -> StructureCompareResult {
    let missing_in_dir_a: Vec<String> =
        remove_subdirectories(&dir_b_content.difference(&dir_a_content).cloned().collect());
    let missing_in_dir_b: Vec<String> =
        remove_subdirectories(&dir_a_content.difference(&dir_b_content).cloned().collect());

    StructureCompareResult {
        missing_in_dir_a,
        missing_in_dir_b,
    }
}

fn main_old() {
    let dir_a_content =
        get_directory_content_recursively(&String::from("./test/08_file_and_directory/dirA"));
    let dir_b_content =
        get_directory_content_recursively(&String::from("./test/08_file_and_directory/dirB"));

    let result = analyze(&dir_a_content, &dir_b_content);
    dbg!("result 02", result);

    let content_compare_result = compare_file_contents(
        &dir_a_content,
        &dir_b_content,
        &String::from("./test/08_file_and_directory/dirA"),
        &String::from("./test/08_file_and_directory/dirB"),
    );
    dbg!("result 02", content_compare_result);
}

fn main() {
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]

mod tests {
    use super::*;

    fn call_structure_compare(path: &str) -> StructureCompareResult {
        analyze(
            &get_directory_content_recursively(&(String::from("./test/") + path + "/dirA")),
            &get_directory_content_recursively(&(String::from("./test/") + path + "/dirB")),
        )
    }
    fn call_content_compare(path: &str) -> ContentCompareResult {
        let path_a = String::from("./test/") + path + "/dirA";
        let path_b = String::from("./test/") + path + "/dirB";
        compare_file_contents(
            &get_directory_content_recursively(&path_a),
            &get_directory_content_recursively(&path_b),
            &path_a,
            &path_b,
        )
    }
    #[test]
    fn t_01_test_files_match() {
        assert_eq!(
            call_structure_compare("01_test_files_match"),
            StructureCompareResult {
                missing_in_dir_a: Vec::new(),
                missing_in_dir_b: Vec::new()
            }
        );
    }

    #[test]
    fn t_02_dir_a_lacks_file() {
        assert_eq!(
            call_structure_compare("02_dirA_lacks_file"),
            StructureCompareResult {
                missing_in_dir_a: [String::from("file2.txt")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_03_dir_b_lacks_file() {
        assert_eq!(
            call_structure_compare("03_dirB_lacks_file"),
            StructureCompareResult {
                missing_in_dir_a: [].to_vec(),
                missing_in_dir_b: [String::from("file1.txt")].to_vec(),
            }
        );
    }

    #[test]
    fn t_04_dir_a_lacks_sub_directory() {
        assert_eq!(
            call_structure_compare("04_dirA_lacks_sub_directory"),
            StructureCompareResult {
                missing_in_dir_a: [String::from("subdir2")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_05_dir_a_lacks_file_in_sub_directory() {
        assert_eq!(
            call_structure_compare("05_dirA_lacks_file_in_sub_directory"),
            StructureCompareResult {
                missing_in_dir_a: [String::from("subdir2/file2.txt")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_06_different_text_content() {
        assert_eq!(
            call_content_compare("06_different_text_content"),
            ContentCompareResult {
                differing_content: [String::from("file1.txt")].to_vec(),
                file_and_directory: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_07_different_binary_content() {
        assert_eq!(
            call_content_compare("07_different_binary_content"),
            ContentCompareResult {
                differing_content: [String::from("file1.jpeg")].to_vec(),
                file_and_directory: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_08_file_and_directory() {
        assert_eq!(
            call_content_compare("08_file_and_directory"),
            ContentCompareResult {
                differing_content: [].to_vec(),
                file_and_directory: [String::from("file1.txt")].to_vec(),
            }
        );
    }
}
