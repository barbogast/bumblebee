use itertools::Itertools;
use std::collections::HashSet;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, PartialOrd)]
struct Missing {
    missing_in_dir_a: Vec<String>,
    missing_in_dir_b: Vec<String>,
}

fn get_directory_content_recursively(dir: String) -> HashSet<String> {
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

fn analyze(dir_a_content: HashSet<String>, dir_b_content: HashSet<String>) -> Missing {
    let missing_in_dir_a: Vec<String> = dir_b_content.difference(&dir_a_content).cloned().collect();
    let missing_in_dir_b: Vec<String> = dir_a_content.difference(&dir_b_content).cloned().collect();

    Missing {
        missing_in_dir_a: remove_subdirectories(&missing_in_dir_a),
        missing_in_dir_b: remove_subdirectories(&missing_in_dir_b),
    }
}

fn main() {
    let dir_a_content =
        get_directory_content_recursively(String::from("./test/02_dirA_lacks_file/dirA"));
    let dir_b_content =
        get_directory_content_recursively(String::from("./test/02_dirA_lacks_file/dirB"));

    let result = analyze(dir_a_content, dir_b_content);
    dbg!("result 02", result);
}

#[cfg(test)]

mod tests {
    use super::*;

    fn run_test(path: &str) -> Missing {
        analyze(
            get_directory_content_recursively(String::from("./test/") + path + "/dirA"),
            get_directory_content_recursively(String::from("./test/") + path + "/dirB"),
        )
    }
    #[test]
    fn t_01_test_files_match() {
        assert_eq!(
            run_test("01_test_files_match"),
            Missing {
                missing_in_dir_a: Vec::new(),
                missing_in_dir_b: Vec::new()
            }
        );
    }

    #[test]
    fn t_02_dir_a_lacks_file() {
        assert_eq!(
            run_test("02_dirA_lacks_file"),
            Missing {
                missing_in_dir_a: [String::from("file2.txt")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_03_dir_b_lacks_file() {
        assert_eq!(
            run_test("03_dirB_lacks_file"),
            Missing {
                missing_in_dir_a: [].to_vec(),
                missing_in_dir_b: [String::from("file1.txt")].to_vec(),
            }
        );
    }

    #[test]
    fn t_04_dir_a_lacks_sub_directory() {
        assert_eq!(
            run_test("04_dirA_lacks_sub_directory"),
            Missing {
                missing_in_dir_a: [String::from("subdir2")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_05_dir_a_lacks_file_in_sub_directory() {
        assert_eq!(
            run_test("05_dirA_lacks_file_in_sub_directory"),
            Missing {
                missing_in_dir_a: [String::from("subdir2/file2.txt")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }
}
