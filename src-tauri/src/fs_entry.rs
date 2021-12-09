#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorEntry {
    pub path: Option<String>,
    pub size: Option<u64>,
    pub content: Option<Vec<FsEntry>>,
    pub reason: String,
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirEntry {
    path: String,
    size: u64,
    number_of_files: u64,
    content: Vec<FsEntry>,
}

impl DirEntry {
    pub fn new(path: String, entries: Vec<FsEntry>) -> Self {
        let size: u64 = entries.iter().map(|entry| entry.size()).sum();
        let number_of_files = entries.iter().map(|entry| entry.number_of_files()).sum();

        Self {
            path,
            content: entries,
            size,
            number_of_files,
        }
    }

    /// Iterator over self.contents which returns only FileEntry entries
    pub fn iter_files(&self) -> impl Iterator<Item = &FsEntry> {
        self.content
            .iter()
            .filter(|entry| matches!(entry, FsEntry::File(_)))
    }

    /// Clone the current entry with up to `levels_to_keep` depth of its contents
    pub fn clone_flat(&self, levels_to_keep: i32) -> Self {
        let content = if levels_to_keep > 0 {
            self.content
                .iter()
                .map(|entry| match entry {
                    FsEntry::File(f) => FsEntry::File(f.clone()),
                    FsEntry::Error(e) => FsEntry::Error(e.clone()),
                    FsEntry::Dir(d) => FsEntry::Dir(d.clone_flat(levels_to_keep - 1)),
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
    pub fn get_entry_by_path(&self, path: String) -> Option<&Self> {
        for entry in &self.content {
            if let FsEntry::Dir(d) = entry {
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

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum FsEntry {
    File(FileEntry),
    Dir(DirEntry),
    Error(ErrorEntry),
}

impl FsEntry {
    pub fn size(&self) -> u64 {
        match self {
            FsEntry::File(f) => f.size,
            FsEntry::Dir(d) => d.size,
            FsEntry::Error(_) => 0,
        }
    }

    pub fn number_of_files(&self) -> u64 {
        match self {
            FsEntry::File(_) => 1,
            FsEntry::Error(_) => 1,
            FsEntry::Dir(d) => d.number_of_files,
        }
    }
}
