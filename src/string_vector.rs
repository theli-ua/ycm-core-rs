use cpython::{py_class, PyResult};
use std::cell::RefCell;

py_class!(pub class StringVector |py| {
    data v: RefCell<Vec<String>>;
    def __new__(_cls) -> PyResult<StringVector> {
        StringVector::create_instance(py, RefCell::new(Vec::default()))
    }
    def __len__(&self) -> PyResult<usize> {
        Ok(self.v(py).borrow().len())
    }
    def __length_hint__(&self) -> PyResult<usize> {
        self.__len__(py)
    }
    def __getitem__(&self, key: usize) -> PyResult<String> {
        Ok(self.v(py).borrow().get(key).unwrap().to_owned())
    }
    def __setitem__(&self, key: usize, value: String) -> PyResult<()> {
        self.v(py).borrow_mut()[key] = value;
        Ok(())
    }
    def __delitem__(&self, key: usize) -> PyResult<()> {
        self.v(py).borrow_mut().remove(key);
        Ok(())
    }
    def __reversed__(&self) -> PyResult<Vec<String>> {
        unimplemented!()
    }
    def __contains__(&self, item: String) -> PyResult<bool> {
        Ok(self.v(py).borrow().contains(&item))
    }
    def append(&self, item: String) -> PyResult<bool> {
        self.v(py).borrow_mut().push(item);
        Ok(true)
    }
});
