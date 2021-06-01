#![allow(non_snake_case)]

use cpython::{py_fn, py_module_initializer, PyList, PyResult, PyString, Python};

mod filter;
mod identifier_completer;
mod string_vector;

use filter::filter_and_sort_candidates;

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
                candidate_property: PyString,
                query: String,
                max_candidates: usize,
            )
        ),
    )?;
    m.add_class::<identifier_completer::IdentifierCompleter>(py)?;
    m.add_class::<string_vector::StringVector>(py)?;
    Ok(())
});

fn has_clang_support(_py: Python) -> PyResult<bool> {
    Ok(false)
}

fn ycm_core_version(_py: Python) -> PyResult<i32> {
    Ok(0)
}

