use std::cell::RefCell;
use std::rc::Rc;

// This macro creates a new mutable rc reference
#[macro_export]
macro_rules! create_mut_rc {
    ($data:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($data))
    };
}

pub type MutRc<T> = Rc<RefCell<T>>;