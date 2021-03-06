use crate::debounce::Debounce;
use crate::fs_entry::{DirEntry, ErrorEntry, FileEntry, FsEntry};
use std::path::Path;
use std::sync::{atomic, Arc, Mutex};
use std::time::Duration;
use std::{fs, io};
use tauri::Manager;

pub struct SavedAnalysisResult(pub Arc<Mutex<Option<FsEntry>>>);

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
struct ProgressPayload {
    path: String,
    number_of_files_found: u64,
    total_size_found: u64,
}

struct Context<'a, 'b> {
    report_progress: &'a mut Debounce<'a, ProgressPayload>,
    should_abort: &'b ShouldAbort,
    number_of_files_found: u64,
    total_size_found: u64,
}

fn analyse_entry(
    context: &mut Context,
    entry: Result<fs::DirEntry, io::Error>,
) -> Result<FsEntry, FsEntry> {
    let entry = entry.map_err(|err| {
        FsEntry::Error(ErrorEntry {
            path: None,
            size: None,
            content: None,
            reason: err.to_string(),
        })
    })?;

    let metadata = entry.metadata().map_err(|err| {
        FsEntry::Error(ErrorEntry {
            path: Some(entry.path().to_string_lossy().to_string()),
            size: None,
            content: None,
            reason: err.to_string(),
        })
    })?;

    if metadata.is_file() {
        Ok(FsEntry::File(FileEntry {
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

fn analyze_directory_recursive<P: AsRef<Path>>(
    context: &mut Context,
    directory_path: P,
) -> FsEntry {
    let path_str = directory_path.as_ref().to_string_lossy().to_string();
    if context.should_abort.0.load(atomic::Ordering::Relaxed) {
        return FsEntry::Error(ErrorEntry {
            path: Some(path_str),
            size: None,
            content: None,
            reason: "Aborted".to_string(),
        });
    }
    context.report_progress.maybe_run(ProgressPayload {
        path: path_str.clone(),
        number_of_files_found: context.number_of_files_found,
        total_size_found: context.total_size_found,
    });

    let read_dir = fs::read_dir(directory_path);
    if let Err(err) = read_dir {
        return FsEntry::Error(ErrorEntry {
            path: Some(path_str),
            size: None,
            content: None,
            reason: err.to_string(),
        });
    }

    let read_dir = read_dir.unwrap();

    let mut entries: Vec<FsEntry> = Vec::new();
    for entry in read_dir {
        match analyse_entry(context, entry) {
            Ok(e) | Err(e) => entries.push(e),
        }
    }

    let dir = DirEntry::new(path_str, entries);
    context.number_of_files_found += dir.iter_files().count() as u64;
    context.total_size_found += dir.iter_files().map(|entry| entry.size()).sum::<u64>();

    FsEntry::Dir(dir)
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyseResult {
    result: FsEntry,
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
            total_size_found: 0,
        },
        Path::new(&path),
    );
    let duration = now.elapsed().as_millis();

    if should_abort.0.load(atomic::Ordering::Relaxed) {
        // No sense to send the data collected so far, return an empty result
        return AnalyseResult {
            result: FsEntry::Error(ErrorEntry {
                path: Some(path),
                size: None,
                content: None,
                reason: "Aborted".to_string(),
            }),
            duration: duration as u64,
        };
    }

    let flat_result = match result {
        FsEntry::Dir(ref d) => FsEntry::Dir(d.clone_flat(2)),
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
) -> Option<FsEntry> {
    let root_entry = &*saved_result.0.lock().unwrap();
    if let Some(FsEntry::Dir(d)) = root_entry {
        return Some(FsEntry::Dir(d.get_entry_by_path(path)?.clone_flat(2)));
    }

    None
}
