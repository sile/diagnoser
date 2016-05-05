use std::io::Read;
use std::io::Result as IoResult;

pub struct Module;

impl Module {
    pub fn from_reader<R: Read>(_input: R) -> IoResult<Self> {
        unimplemented!()
    }
}
