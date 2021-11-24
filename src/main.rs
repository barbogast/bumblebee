use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

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

#[derive(Debug)]
struct Missing {
    missing_in_dir_a: HashSet<String>,
    missing_in_dir_b: HashSet<String>,
}

fn analyze(dir_a_content: HashSet<String>, dir_b_content: HashSet<String>) -> Missing {
    Missing {
        missing_in_dir_a: dir_b_content.difference(&dir_a_content).cloned().collect(),
        missing_in_dir_b: dir_a_content.difference(&dir_b_content).cloned().collect(),
    }
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
