#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorEntry {
    pub path: Option<String>,
    pub size: Option<u64>,
    pub content: Option<Vec<Entry>>,
    pub reason: String,
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirEntry {
    pub path: String,
    pub size: u64,
    pub number_of_files: u64,
    pub content: Vec<Entry>,
}

impl DirEntry {
    /// Clone the current entry with up to `levels_to_keep` depth of its contents
    pub fn clone_flat(&self, levels_to_keep: i32) -> Self {
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
    pub fn get_entry_by_path(&self, path: String) -> Option<&Self> {
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

#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    File(FileEntry),
    Dir(DirEntry),
    Error(ErrorEntry),
}

impl Entry {
    pub fn size(&self) -> u64 {
        match self {
            Entry::File(f) => f.size,
            Entry::Dir(d) => d.size,
            Entry::Error(_) => 0,
        }
    }

    pub fn number_of_files(&self) -> u64 {
        match self {
            Entry::File(_) => 1,
            Entry::Error(_) => 1,
            Entry::Dir(d) => d.number_of_files,
        }
    }
}
