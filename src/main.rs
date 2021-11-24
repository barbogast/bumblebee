use itertools::Itertools;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

fn read_directories() -> io::Result<(
    std::vec::Vec<std::path::PathBuf>,
    std::vec::Vec<std::path::PathBuf>,
)> {
    let entries1 = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let entries2 = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok((entries1, entries2))
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, cb: &dyn Fn(&std::fs::DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn xx(dir_entry: &std::fs::DirEntry) {}

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

#[derive(Debug, PartialEq, PartialOrd)]
struct Missing {
    missing_in_dir_a: Vec<String>,
    missing_in_dir_b: Vec<String>,
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

fn run_test(path: &str) -> Missing {
    analyze(
        get_directory_content_recursively(String::from("./test/") + path + "/dirA"),
        get_directory_content_recursively(String::from("./test/") + path + "/dirB"),
    )
}

fn main() {
    println!("Hello, world!");
    match read_directories() {
        Err(why) => {
            println!("! {:?}", why.kind());
            return ();
        }
        Ok(folder_entries) => {
            dbg!(folder_entries)
        }
    };

    match visit_dirs(Path::new("./test/test_files_match"), &xx) {
        Ok(res) => println!("! {:?}", res),
        Err(why) => println!("! {:?}", why.kind()),
    }

    let dirA_content =
        get_directory_content_recursively(String::from("./test/01_test_files_match/dirA"));
    let dirB_content =
        get_directory_content_recursively(String::from("./test/01_test_files_match/dirB"));

    let result = analyze(dirA_content, dirB_content);
    dbg!("result 01", result);

    let dirA_content =
        get_directory_content_recursively(String::from("./test/02_dirA_lacks_file/dirA"));
    let dirB_content =
        get_directory_content_recursively(String::from("./test/02_dirA_lacks_file/dirB"));

    let result = analyze(dirA_content, dirB_content);
    dbg!("result 02", result);

    let dirA_content =
        get_directory_content_recursively(String::from("./test/04_dirA_lacks_sub_directory/dirA"));
    let dirB_content =
        get_directory_content_recursively(String::from("./test/04_dirA_lacks_sub_directory/dirB"));

    let result = analyze(dirA_content, dirB_content);
    dbg!("result 04", result);

    // for entry in dirA_content {
    // }

    // match fs::read_dir("./test").and(fs::read_dir("./test/test_files_match")) {
    // match fs::read_dir("./test").unwrap_() {
    //     Err(why) => println!("! {:?}", why.kind()),
    //     Ok(paths) => {
    //         for path in paths {
    //             println!("> {:?}", path.unwrap().path());
    //         }
    //     }
    // }
    // match fs::read_dir("../bumblebee") {
    //     Err(why) => println!("! {:?}", why.kind()),
    //     Ok(paths) => {
    //         for path in paths {
    //             println!("> {:?}", path.unwrap().path());
    //         }
    //     }
    // }
}

#[cfg(test)]

mod tests {
    use super::*;
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
    fn t_02_dirA_lacks_file() {
        assert_eq!(
            run_test("02_dirA_lacks_file"),
            Missing {
                missing_in_dir_a: [String::from("file2.txt")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_03_dirB_lacks_file() {
        assert_eq!(
            run_test("03_dirB_lacks_file"),
            Missing {
                missing_in_dir_a: [].to_vec(),
                missing_in_dir_b: [String::from("file1.txt")].to_vec(),
            }
        );
    }

    #[test]
    fn t_04_dirA_lacks_sub_directory() {
        assert_eq!(
            run_test("04_dirA_lacks_sub_directory"),
            Missing {
                missing_in_dir_a: [String::from("subdir2")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }

    #[test]
    fn t_05_dirA_lacks_file_in_sub_directory() {
        assert_eq!(
            run_test("05_dirA_lacks_file_in_sub_directory"),
            Missing {
                missing_in_dir_a: [String::from("subdir2/file2.txt")].to_vec(),
                missing_in_dir_b: [].to_vec(),
            }
        );
    }
}
