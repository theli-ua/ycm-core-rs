use partial_sort::PartialSort;

use cpython::ObjectProtocol;
use cpython::{PyBytes, PyDict, PyList, PyObject, PyResult, PyString, PyUnicode, Python};

// I am not sure what exactly needs to happen here
fn get_ut8_string<'a>(py: Python, s: &PyObject) -> String {
    if let Ok(s) = s.cast_as::<PyUnicode>(py) {
        s.to_string(py).unwrap().to_string()
    } else if let Ok(s) = s.cast_as::<PyBytes>(py) {
        String::from_utf8_lossy(s.data(py)).to_string()
    } else {
        s.str(py).unwrap().to_string_lossy(py).to_string()
    }
}

fn candidates_from_objlist(
    py: Python,
    candidates: &PyList,
    candidate_property: &PyString,
) -> Vec<String> {
    if candidate_property.to_string_lossy(py).is_empty() {
        candidates
            .iter(py)
            .map(|o| get_ut8_string(py, &o))
            .collect()
    } else {
        candidates
            .iter(py)
            .map(|o| {
                let prop = o
                    .cast_as::<PyDict>(py)
                    .unwrap()
                    .get_item(py, &candidate_property)
                    .unwrap();
                get_ut8_string(py, &prop)
            })
            .collect()
    }
}

pub fn filter_and_sort_candidates(
    py: Python,
    candidates: PyList,
    candidate_property: PyString,
    query: String,
    max_candidates: usize,
) -> PyResult<PyList> {
    let candidates_str = candidates_from_objlist(py, &candidates, &candidate_property);
    let mut filtered_candidates = candidates_str
        .into_iter()
        .enumerate()
        .filter_map(|(i, candidate)| {
            if candidate.find(&query).is_some() {
                Some((i, candidate))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    filtered_candidates.partial_sort(max_candidates, |a, b| a.1.cmp(&b.1));
    filtered_candidates.resize(max_candidates, Default::default());

    Ok(PyList::new(
        py,
        &filtered_candidates
            .into_iter()
            .map(|(i, _)| candidates.get_item(py, i))
            .collect::<Vec<_>>(),
    ))
}
