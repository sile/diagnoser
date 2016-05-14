// http://erlang.org/doc/apps/erts/erl_ext_dist.html
use std::io::Read;
use std::io::Result as IoResult;
use beam::term::Term;

pub fn from_reader<R: Read>(reader: R) -> IoResult<Term> {
    unimplemented!()
}
