use std::{
    collections::VecDeque,
    fs, io,
    path::{Path, PathBuf},
};

/// # Description
/// iterative BFS traversal for objects in the filesystem
/// ## Comments
/// standard recursive(DFS) method of traversal would cause a stack overflow if the directory tree gets too deep
/// so iterative method is preferred, especially for a CLI application
pub struct FileSystemIterator {
    path_queue: VecDeque<PathBuf>,
}
impl FileSystemIterator {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path_queue: vec![path.as_ref().to_path_buf()]
                .into_iter()
                .collect::<VecDeque<_>>(),
        }
    }
}
impl Iterator for FileSystemIterator {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        self.path_queue.pop_front().map(|path| {
            //add adjacent nodes to the queue
            if let Ok(iterator) = fs::read_dir(&path) {
                for entry in iterator.filter_map(|a| a.ok()) {
                    let path = entry.path();
                    self.path_queue.push_back(path);
                }
            }
            path
        })
    }
}

#[allow(dead_code)]
pub fn collect_files_iterative<P, PRED>(path: P, predicate: PRED) -> Vec<PathBuf>
where
    PRED: Fn(&PathBuf) -> bool,
    P: AsRef<Path>,
{
    //visited table not needed because filesystems have no cycles
    let mut path_queue = VecDeque::new();
    path_queue.push_back(path.as_ref().to_path_buf());
    let mut file_list = vec![];

    while let Some(path) = path_queue.pop_front() {
        //add adjacent nodes to the queue
        if let Ok(iterator) = fs::read_dir(&path) {
            for entry in iterator.filter_map(|a| a.ok()) {
                let path = entry.path();
                path_queue.push_back(path);
            }
        }

        //check if the file what were looking for before returning the path
        if path.is_file() && predicate(&path) {
            file_list.push(path);
        }
    }
    file_list
}

#[allow(dead_code)]
/// # Description
/// this way of traversing the file is not efficent and will cause a stack overflow if file tree is too deep
pub fn collect_files<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
    if path.as_ref().is_file() {
        return Ok(vec![path.as_ref().to_path_buf()]);
    }
    let mut files = vec![];
    for object in fs::read_dir(path)? {
        let entry = object?;
        let path = entry.path();
        let res = collect_files(path)?;
        for x in res {
            files.push(x);
        }
    }
    Ok(files)
}