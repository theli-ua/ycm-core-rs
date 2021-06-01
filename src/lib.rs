#![allow(non_snake_case)]

use cpython::{
    py_class, py_fn, py_module_initializer, FromPyObject, PyList, PyNone, PyResult, Python,
    ToPyObject,
};

mod identifier_completer;
mod string_vector;

py_module_initializer!(ycm_core, |py, m| {
    m.add(py, "__doc__", "This module is implemented in Rust.")?;
    m.add(py, "HasClangSupport", py_fn!(py, has_clang_support()))?;
    m.add(py, "YcmCoreVersion", py_fn!(py, ycm_core_version()))?;
    m.add(
        py,
        "FilterAndSortCandidates",
        py_fn!(
            py,
            filter_and_sort_candidates(
                candidates: PyList,
                candidate_property: &str,
                query: String,
                max_candidates: usize,
            )
        ),
    )?;
    m.add_class::<IdentifierCompleter>(py)?;
    m.add_class::<string_vector::StringVector>(py)?;
    Ok(())
});

py_class!(class IdentifierCompleter |py| {
    def __new__(_cls, _arg: i32) -> PyResult<IdentifierCompleter> {
        unimplemented!();
    }
    def AddIdentifiersToDatabase(&self, _new_candidates: Vec<String>, _filetype: &str, _filepath: &str) -> PyResult<PyNone> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
    def ClearForFileAndAddIdentifiersToDatabase(&self, _new_candidates: Vec<String>, _filetype: &str, _filepath: &str) -> PyResult<PyNone> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
    def AddIdentifiersToDatabaseFromTagFiles(&self, _absolute_paths_to_tag_files: Vec<String>) -> PyResult<PyNone> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
    def CandidatesForQueryAndType(&self, _query: String, _filetype: &str, _max_candidates: usize) -> PyResult<Vec<String>> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
});

fn has_clang_support(_py: Python) -> PyResult<bool> {
    Ok(false)
}

fn ycm_core_version(_py: Python) -> PyResult<i32> {
    Ok(0)
}

fn filter_and_sort_candidates(
    _py: Python,
    _candidates: PyList,
    _candidate_property: &str,
    _query: String,
    _max_candidates: usize,
) -> PyResult<bool> {
    unimplemented!()
}
