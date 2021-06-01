use crate::string_vector::StringVector;
use cpython::{py_class, PyNone, PyResult};

py_class!(pub class IdentifierCompleter |py| {
    def __new__(_cls, _arg: i32) -> PyResult<IdentifierCompleter> {
        unimplemented!();
    }
    def AddIdentifiersToDatabase(&self, _new_candidates: &StringVector, _filetype: &str, _filepath: &str) -> PyResult<PyNone> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
    def ClearForFileAndAddIdentifiersToDatabase(&self, _new_candidates: &StringVector, _filetype: &str, _filepath: &str) -> PyResult<PyNone> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
    def AddIdentifiersToDatabaseFromTagFiles(&self, _absolute_paths_to_tag_files: &StringVector) -> PyResult<PyNone> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
    def CandidatesForQueryAndType(&self, _query: String, _filetype: &str, _max_candidates: usize) -> PyResult<StringVector> {
        py.allow_threads(||{
            unimplemented!()
        })
    }
});

