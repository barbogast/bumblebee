use crate::debounce::Debounce;
use std::path::Path;
use std::sync::{atomic, Arc, Mutex};
use std::time::Duration;
use std::{fs, io};
use tauri::Manager;

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirEntry {
    path: String,
    size: u64,
    number_of_files: u64,
    content: Vec<Entry>,
}

impl DirEntry {
    /// Clone the current entry with up to `levels_to_keep` depth of its contents
    fn clone_flat(&self, levels_to_keep: i32) -> Self {
        let content = if levels_to_keep > 0 {
            self.content
                .iter()
                .map(|entry| match entry {
                    Entry::File(f) => Entry::File(f.clone()),
                    Entry::Error(e) => Entry::Error(e.clone()),
                    Entry::Dir(d) => Entry::Dir(d.clone_flat(levels_to_keep - 1)),
                })
                .collect()
        } else {
            vec![]
        };
        DirEntry {
            content,
            path: self.path.clone(),
            size: self.size,
            number_of_files: self.number_of_files,
        }
    }

    /// Search for an entry recursivly within the current entry
    fn get_entry_by_path(&self, path: String) -> Option<&Self> {
        for entry in &self.content {
            if let Entry::Dir(d) = entry {
                dbg!("entry", &d.path);
                if d.path == path {
                    return Some(d);
                } else if path.starts_with(&d.path) {
                    return d.get_entry_by_path(path);
                };
            }
        }
        None
    }
}

pub struct SavedAnalysisResult(pub Arc<Mutex<Option<Entry>>>);

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorEntry {
    path: Option<String>,
    size: Option<u64>,
    content: Option<Vec<Entry>>,
    reason: String,
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    path: String,
    size: u64,
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    File(FileEntry),
    Dir(DirEntry),
    Error(ErrorEntry),
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
struct ProgressPayload {
    path: String,
    number_of_files_found: u64,
}

struct Context<'a, 'b> {
    report_progress: &'a mut Debounce<'a, ProgressPayload>,
    should_abort: &'b ShouldAbort,
    number_of_files_found: u64,
}

impl Entry {
    fn size(&self) -> u64 {
        match self {
            Entry::File(f) => f.size,
            Entry::Dir(d) => d.size,
            Entry::Error(_) => 0,
        }
    }

    fn number_of_files(&self) -> u64 {
        match self {
            Entry::File(_) => 1,
            Entry::Error(_) => 1,
            Entry::Dir(d) => d.number_of_files,
        }
    }
}

fn analyse_entry(
    context: &mut Context,
    entry: Result<fs::DirEntry, io::Error>,
) -> Result<Entry, Entry> {
    let entry = entry.map_err(|err| {
        Entry::Error(ErrorEntry {
            path: None,
            size: None,
            content: None,
            reason: err.to_string(),
        })
    })?;

    let metadata = entry.metadata().map_err(|err| {
        Entry::Error(ErrorEntry {
            path: Some(entry.path().to_string_lossy().to_string()),
            size: None,
            content: None,
            reason: err.to_string(),
        })
    })?;

    if metadata.is_file() {
        Ok(Entry::File(FileEntry {
            path: entry.path().to_string_lossy().to_string(),
            size: metadata.len(),
        }))

        // TODO: is_symlink is unstable
        // } else if metaadata.is_symlink() {
    } else {
        // TODO: Implement a limit for the recursion depth to protect against a stack overflow
        Ok(analyze_directory_recursive(context, entry.path()))
    }
}

fn analyze_directory_recursive<P: AsRef<Path>>(context: &mut Context, directory_path: P) -> Entry {
    let path_str = directory_path.as_ref().to_string_lossy().to_string();
    if context.should_abort.0.load(atomic::Ordering::Relaxed) {
        return Entry::Error(ErrorEntry {
            path: Some(path_str),
            size: None,
            content: None,
            reason: "Aborted".to_string(),
        });
    }
    context.report_progress.maybe_run(ProgressPayload {
        path: path_str.clone(),
        number_of_files_found: context.number_of_files_found,
    });

    let read_dir = fs::read_dir(directory_path);
    if let Err(err) = read_dir {
        return Entry::Error(ErrorEntry {
            path: Some(path_str),
            size: None,
            content: None,
            reason: err.to_string(),
        });
    }

    let read_dir = read_dir.unwrap();

    let mut entries: Vec<Entry> = Vec::new();
    for entry in read_dir {
        match analyse_entry(context, entry) {
            Ok(e) | Err(e) => entries.push(e),
        }
    }

    let size: u64 = entries.iter().map(|entry| entry.size()).sum();
    let number_of_files = entries.iter().map(|entry| entry.number_of_files()).sum();
    context.number_of_files_found += entries
        .iter()
        .filter(|entry| matches!(entry, Entry::File(_)))
        .count() as u64;

    Entry::Dir(DirEntry {
        path: path_str,
        content: entries,
        size,
        number_of_files,
    })
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
    saved_result: tauri::State<SavedAnalysisResult>,
    path: String,
) -> AnalyseResult {
    should_abort.0.store(false, atomic::Ordering::Relaxed);
    use std::time::Instant;
    let now = Instant::now();
    let func = |payload| app_handle.emit_all("progress", payload).unwrap();
    let mut report_progress = Debounce::new(Duration::from_millis(100), &func);
    let result = analyze_directory_recursive(
        &mut Context {
            report_progress: &mut report_progress,
            should_abort: &should_abort,
            number_of_files_found: 0,
        },
        Path::new(&path),
    );
    let duration = now.elapsed().as_millis();

    if should_abort.0.load(atomic::Ordering::Relaxed) {
        // No sense to send the data collected so far, return an empty result
        return AnalyseResult {
            result: Entry::Error(ErrorEntry {
                path: Some(path),
                size: None,
                content: None,
                reason: "Aborted".to_string(),
            }),
            duration: duration as u64,
        };
    }

    let flat_result = match result {
        Entry::Dir(ref d) => Entry::Dir(d.clone_flat(2)),
        _ => panic!(),
    };

    *saved_result.0.lock().unwrap() = Some(result);

    AnalyseResult {
        result: flat_result,
        duration: duration as u64,
    }
}
#[derive(Debug)]
pub struct ShouldAbort(pub atomic::AtomicBool);

#[tauri::command]
pub fn abort(should_abort: tauri::State<'_, ShouldAbort>) {
    should_abort.0.store(true, atomic::Ordering::Relaxed);
}

#[tauri::command]
pub fn load_nested_directory(
    path: String,
    saved_result: tauri::State<'_, SavedAnalysisResult>,
) -> Option<Entry> {
    let root_entry = &*saved_result.0.lock().unwrap();
    if let Some(Entry::Dir(d)) = root_entry {
        return Some(Entry::Dir(d.get_entry_by_path(path)?.clone_flat(2)));
    }

    None
}
