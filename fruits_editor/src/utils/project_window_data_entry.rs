use std::ffi::OsString;

#[derive(Debug, Clone)]
pub struct ProjectWindowDataEntry {
    pub name: String,
    pub path: OsString,
    pub children: Vec<ProjectWindowDataEntry>,
}

impl ProjectWindowDataEntry {
    pub fn scan(src: &std::path::PathBuf) -> Self {
        Self {
            name: src.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default(),
            path: src.as_os_str().to_os_string(),
            children: Self::get_children(src),
        }
    }

    fn from_dir_entry(src: &std::fs::DirEntry) -> Self {
        Self {
            name: src.file_name().to_string_lossy().into_owned(),
            path: src.path().as_os_str().to_os_string(),
            children: Self::get_children(&src.path()),
        }
    }

    fn get_children(src: &std::path::PathBuf) -> Vec<Self> {
        let mut result = Vec::new();

        let Ok(read_dir) = std::fs::read_dir(src) else {
            return result;
        };

        for dir_entry in read_dir {
            if let Ok(dir_entry) = dir_entry {
                result.push(Self::from_dir_entry(&dir_entry));
            }
        }

        result
    }
}
