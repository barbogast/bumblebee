use crate::debounce::Debounce;
use std::path::Path;
use std::sync::atomic;
use std::time::Duration;
use std::{fs, io};
use tauri::Manager;

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum Entry {
    File {
        path: String,
        size: u64,
    },
    Dir {
        path: String,
        size: u64,
        number_of_files: u64,
        content: Vec<Entry>,
    },
    Error {
        path: Option<String>,
        size: Option<u64>,
        content: Option<Vec<Entry>>,
        reason: String,
    },
}

struct Context<'a, 'b> {
    report_progress: &'a mut Debounce<'a, String>,
    should_abort: &'b ShouldAbort,
}

impl Entry {
    fn size(&self) -> u64 {
        match self {
            Entry::File { size, .. } => *size,
            Entry::Dir { size, .. } => *size,
            Entry::Error { .. } => 0,
        }
    }

    fn number_of_files(&self) -> u64 {
        match self {
            Entry::File { .. } => 1,
            Entry::Error { .. } => 1,
            Entry::Dir {
                number_of_files, ..
            } => *number_of_files,
        }
    }
}

fn analyse_entry(context: &mut Context, entry: Result<fs::DirEntry, io::Error>) -> Entry {
    if let Err(err) = entry {
        return Entry::Error {
            path: None,
            size: None,
            content: None,
            reason: err.to_string(),
        };
    }

    let entry = entry.unwrap();

    let metadata = entry.metadata();
    if let Err(err) = metadata {
        return Entry::Error {
            path: Some(entry.path().to_string_lossy().to_string()),
            size: None,
            content: None,
            reason: err.to_string(),
        };
    }
    let metadata = metadata.unwrap();

    if metadata.is_file() {
        return Entry::File {
            path: entry.path().to_string_lossy().to_string(),
            size: metadata.len(),
        };

        // TODO: is_symlink is unstable
        // } else if metaadata.is_symlink() {
    } else {
        // TODO: Implement a limit for the recursion depth to protect against a stack overflow
        analyze_directory_recursive(context, entry.path())
    }
}

fn analyze_directory_recursive<P: AsRef<Path>>(context: &mut Context, directory_path: P) -> Entry {
    let path_str = directory_path.as_ref().to_string_lossy().to_string();
    if context.should_abort.0.load(atomic::Ordering::Relaxed) {
        return Entry::Error {
            path: Some(path_str),
            size: None,
            content: None,
            reason: "Aborted".to_string(),
        };
    }
    context.report_progress.maybe_run(path_str.clone());

    let read_dir = fs::read_dir(directory_path);
    if let Err(err) = read_dir {
        return Entry::Error {
            path: Some(path_str),
            size: None,
            content: None,
            reason: err.to_string(),
        };
    }

    let read_dir = read_dir.unwrap();

    let mut entries: Vec<Entry> = Vec::new();
    for entry in read_dir {
        entries.push(analyse_entry(context, entry));
    }

    let size: u64 = entries.iter().map(|entry| entry.size()).sum();
    let number_of_files = entries.iter().map(|entry| entry.number_of_files()).sum();

    Entry::Dir {
        path: path_str,
        content: entries,
        size,
        number_of_files,
    }
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyseResult {
    result: Entry,
    duration: u64,
}

#[tauri::command(async)]
pub fn analyze_disk_usage(
    app_handle: tauri::AppHandle,
    should_abort: tauri::State<ShouldAbort>,
    path: String,
) -> AnalyseResult {
    should_abort.0.store(false, atomic::Ordering::Relaxed);
    use std::time::Instant;
    let now = Instant::now();
    let func = |path: String| app_handle.emit_all("progress", path).unwrap();
    let mut report_progress = Debounce::new(Duration::from_millis(100), &func);
    let mut result = analyze_directory_recursive(
        &mut Context {
            report_progress: &mut report_progress,
            should_abort: &should_abort,
        },
        Path::new(&path),
    );
    let duration = now.elapsed().as_millis();

    if should_abort.0.load(atomic::Ordering::Relaxed) {
        // No sense to send the data collected so far, return an empty result
        result = Entry::Error {
            path: Some(path),
            size: None,
            content: None,
            reason: "Aborted".to_string(),
        };
    }
    AnalyseResult {
        result,
        duration: duration as u64,
    }
}
#[derive(Debug)]
pub struct ShouldAbort(pub atomic::AtomicBool);

#[tauri::command]
pub fn abort(should_abort: tauri::State<'_, ShouldAbort>) {
    should_abort.0.store(true, atomic::Ordering::Relaxed);
}
