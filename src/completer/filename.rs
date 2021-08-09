use log::debug;
use regex::Regex;
use std::{collections::HashSet, path::PathBuf};

use crate::{
    core::query::filter_and_sort_generic_candidates,
    core::utils,
    ycmd_types::{Candidate, SimpleRequest},
};

use super::{Completer, CompleterInner, CompletionConfig};

use itertools::Itertools;

#[cfg(windows)]
pub const PATH_SEPARATORS: &str = "/\\";
#[cfg(unix)]
pub const PATH_SEPARATORS: &str = "/";

#[cfg(unix)]
const HEAD_PATTERN: &str = r#"\.{1,2}|~|\$[^$]+"#;

#[cfg(windows)]
const HEAD_PATTERN: &str = r#"\.{1,2}|~|\$[^$]+|[A-Za-z]:|%[^%]+%"#;

lazy_static::lazy_static! {
static ref PATH_SEPARATORS_REGEX: Regex = Regex::new(format!("([{0}][^{0}]*|[{0}]$)", PATH_SEPARATORS).as_str())
    .unwrap();
}

pub struct FilenameCompleter {
    config: CompletionConfig,
    blacklist: HashSet<String>,
    use_working_dir: bool,
}

#[derive(PartialEq)]
pub enum FileType {
    File,
    Dir,
    FileAndDir,
    Framework,
    FileAndFramework,
    DirAndFramework,
    FileAndDirAndFramework,
}

impl ToString for FileType {
    fn to_string(&self) -> String {
        match self {
            FileType::File => String::from("[File]"),
            FileType::Dir => String::from("[Dir]"),
            FileType::FileAndDir => String::from("[File&Dir]"),
            FileType::Framework => String::from("[Framework]"),
            FileType::FileAndFramework => String::from("[File&Framework]"),
            FileType::DirAndFramework => String::from("[Dir&Framework]"),
            FileType::FileAndDirAndFramework => String::from("[File&Dir&Framework]"),
        }
    }
}

impl FilenameCompleter {
    pub fn new(
        config: CompletionConfig,
        blacklist: HashSet<String>,
        use_working_dir: bool,
    ) -> Self {
        Self {
            config,
            blacklist,
            use_working_dir,
        }
    }
}

impl FilenameCompleter {
    fn working_directory(&self, working_dir: &Option<PathBuf>, filepath: &PathBuf) -> PathBuf {
        if self.use_working_dir {
            working_dir.clone()
        } else {
            filepath.parent().map(|f| f.to_owned())
        }
        .unwrap_or_else(utils::get_current_dir)
    }

    fn current_filetype_completion_disabled(&self, filetypes: &[String]) -> bool {
        self.blacklist.contains("*") || filetypes.iter().any(|f| self.blacklist.contains(f))
    }

    fn get_dir_head_regex(&self, directory: &str) -> Regex {
        let paths = utils::list_dir(directory);
        #[allow(unstable_name_collisions)]
        let patterns = std::iter::once(HEAD_PATTERN.to_string())
            .chain(paths)
            .intersperse(String::from("|"));
        Regex::new(
            std::iter::once(String::from("("))
                .chain(patterns)
                .chain(std::iter::once(String::from(")$")))
                .collect::<String>()
                .as_str(),
        )
        .unwrap()
    }

    ///Return the tuple (|path|, |start_column|) where |path| is a path that
    ///could be completed on the current line before the cursor and |start_column|
    ///is the column where the completion should start. (None, None) is returned if
    ///no suitable path is found.
    fn search_path(&self, request: &SimpleRequest) -> Option<(PathBuf, usize)> {
        let current_line = request.prefix();
        let mut matches = PATH_SEPARATORS_REGEX
            .find_iter(current_line)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return None;
        }
        let working_dir = self.working_directory(&request.working_dir, &request.filepath);

        let head_regex = self.get_dir_head_regex(working_dir.to_str().unwrap());
        let last_match = dbg!(matches.pop().unwrap());
        let last_match_start = last_match.start();
        let matches_n = matches.len();
        // Go through all path separators from left to right.
        for m in matches {
            // Check if ".", "..", "~", an environment variable, one of the current
            // directories, or a drive letter on Windows match just before the
            // separator. If so, extract the path from the start of the match to the
            // latest path separator. Expand "~" and the environment variables in the
            // path. If the path is relative, convert it to an absolute path relative
            // to the working directory. If the resulting path exists, return it and
            if let Some(head_match) = head_regex.find(&current_line[..m.start()]) {
                let path = &current_line[head_match.start()..last_match_start];
                let path = utils::expand_vars(path);
                let path = std::path::Path::new(&*path);
                let path = if path.is_relative() {
                    working_dir.join(path)
                } else {
                    path.to_owned()
                };

                if path.exists() {
                    return Some((path, last_match_start + 1));
                }
            } else {
                // Otherwise, the path may start with "/" (or "\" on Windows). Extract the
                // path from the current path separator to the latest one. If the path is
                // not empty and does not only consist of path separators, expand "~" and
                // the environment variables in the path. If the resulting path exists,
                // return it and the column just after the latest path separator as the
                // starting column.
                let path = &current_line[m.start()..last_match_start];
                if !path
                    .trim_matches(|c| PATH_SEPARATORS.contains(c))
                    .is_empty()
                {
                    let path = utils::expand_vars(path);
                    let path = std::path::Path::new(&*path);
                    if path.exists() {
                        return Some((path.to_owned(), last_match_start + 1));
                    }
                }
            }
            // No suitable paths have been found after going through all separators. The
            // path could be exactly "/" (or "\" on Windows). Only return the path if
            // there are no other path separators on the line. This prevents always
            // completing the root directory if nothing is matched.
            // TODO: completion on a single "/" or "\" is not really desirable in
            // languages where such characters are part of special constructs like
            // comments in C/C++ or closing tags in HTML. This behavior could be improved
            // by using rules that depend on the filetype.
        }
        if matches_n == 1 {
            return Some((
                std::path::PathBuf::from(&String::from(std::path::MAIN_SEPARATOR)),
                last_match_start + 1,
            ));
        }
        None
    }

    fn generate_path_candidates(&self, dir: PathBuf) -> Vec<Candidate> {
        match std::fs::read_dir(dir) {
            Err(_) => vec![],
            Ok(d) => d
                .map(|f| f.ok())
                .flatten()
                .map(|f| {
                    let name = f.file_name().to_string_lossy().to_string();
                    let file_type = match f.file_type() {
                        Err(_) => FileType::FileAndDir,
                        Ok(t) => {
                            if t.is_dir() {
                                FileType::Dir
                            } else if t.is_file() {
                                FileType::File
                            } else {
                                FileType::FileAndDir
                            }
                        }
                    }
                    .to_string();
                    Candidate {
                        insertion_text: name,
                        extra_menu_info: Some(file_type),
                        menu_text: None,
                        detailed_info: None,
                        kind: None,
                        extra_data: None,
                    }
                })
                .collect(),
        }
    }
}

impl CompleterInner for FilenameCompleter {
    fn get_settings(&self) -> &CompletionConfig {
        &self.config
    }

    fn get_settings_mut(&mut self) -> &mut CompletionConfig {
        &mut self.config
    }
}

impl Completer for FilenameCompleter {
    fn should_use_now(&self, request: &SimpleRequest) -> bool {
        !self.current_filetype_completion_disabled(request.filetypes()) && {
            let s = self.search_path(request);
            debug!("search_path: {:?}", s);
            s.is_some()
        }
    }

    fn compute_candidates(&self, request: &mut SimpleRequest) -> Vec<Candidate> {
        if !self.should_use_now(request) {
            vec![]
        } else if let Some((dir, start)) = self.search_path(request) {
            request.start_column = Some(start);
            let candidates = self.generate_path_candidates(dir);
            debug!("Path completion candidates: {:?}", candidates);
            filter_and_sort_generic_candidates(
                candidates,
                request.query(),
                self.get_settings().max_candidates,
                |c| &c.insertion_text,
            )
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::ycmd_types::FileData;

    use super::*;
    use tempfile::tempdir;
    #[test]
    fn test_search_path_abs() {
        let completer = FilenameCompleter {
            blacklist: HashSet::default(),
            config: CompletionConfig {
                min_num_chars: 1,
                max_diagnostics_to_display: 1,
                completion_triggers: Default::default(),
                signature_triggers: Default::default(),
                max_candidates: 10,
                max_candidates_to_detail: 1,
            },
            use_working_dir: false,
        };
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("candidate.txt");
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "_ was here. Briefly.").unwrap();
        core::mem::drop(file);

        let mut file_data = std::collections::HashMap::default();
        let file_contents = format!("1234{}/ ", tmp.path().display());
        let column_num = file_contents.len() + 1; // on the last space in that line
        file_data.insert(
            PathBuf::from("/file"),
            FileData {
                filetypes: vec![],
                contents: file_contents,
            },
        );
        let request = SimpleRequest {
            line_num: 1,
            column_num,
            filepath: PathBuf::from("/file"),
            file_data,
            completer_target: None,
            working_dir: None,
            extra_conf_data: None,
            start_column: None,
        };
        assert_eq!(
            Some((tmp.into_path(), column_num - 2)),
            completer.search_path(&request)
        );
    }

    #[test]
    fn test_search_path_relative() {
        let completer = FilenameCompleter {
            blacklist: HashSet::default(),
            config: CompletionConfig {
                min_num_chars: 1,
                max_diagnostics_to_display: 1,
                completion_triggers: Default::default(),
                signature_triggers: Default::default(),
                max_candidates: 10,
                max_candidates_to_detail: 1,
            },
            use_working_dir: false,
        };
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("candidate.txt");
        let mut file = File::create(file_path.clone()).unwrap();
        writeln!(file, "_ was here. Briefly.").unwrap();
        core::mem::drop(file);

        let mut file_data = std::collections::HashMap::default();
        let file_contents = format!(
            "123 ../{}/ ",
            tmp.path().file_name().unwrap().to_string_lossy()
        );
        let column_num = file_contents.len() + 1; // on the last space in that line
        file_data.insert(
            file_path.clone(),
            FileData {
                filetypes: vec![],
                contents: file_contents,
            },
        );
        let request = SimpleRequest {
            line_num: 1,
            column_num,
            filepath: file_path,
            file_data,
            completer_target: None,
            working_dir: None,
            extra_conf_data: None,
            start_column: None,
        };
        assert_eq!(
            Some((
                PathBuf::from(format!(
                    "{}/../{}",
                    tmp.path().display(),
                    tmp.path().file_name().unwrap().to_string_lossy()
                )),
                column_num - 2
            )),
            completer.search_path(&request)
        );
    }
}
