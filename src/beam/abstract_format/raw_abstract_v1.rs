use std::io::Result as IoResult;
use beam::term;
use beam::term::Term;

pub struct Module;

impl Module {
    pub fn from_abstract_code(abstract_code: &Term) -> IoResult<Self> {
        use beam::term::RefTerm::*;
        match abstract_code.as_ref_term_level1() {
            Tuple2((Atom("raw_abstract_v1"), List(_, list)), _) => Self::from_forms(list),
            _ => {
                invalid_data_error("First term must be a `{raw_abstract_v1, term()}` format"
                                       .to_string())
            }
        }
    }

    pub fn from_forms(forms: &term::List) -> IoResult<Self> {
        panic!("TODO: {}", forms)
    }
}

fn invalid_data_error<T>(message: String) -> IoResult<T> {
    use std::io::Error;
    use std::io::ErrorKind;
    Err(Error::new(ErrorKind::InvalidData, message))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::PathBuf;
    use beam::external;
    use super::*;

    #[test]
    fn from_term_works() {
        let file = File::open(test_file("hello.beam")).expect("Can't open file");
        let ext_fmt_module = external::module::Module::from_reader(file).expect("Can't parse file");
        let abstract_code = ext_fmt_module.abstract_code.as_ref().unwrap();
        let module = Module::from_abstract_code(abstract_code).unwrap();
    }

    fn test_file(name: &str) -> PathBuf {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.pop();
        path.push("testdata/");
        path.push(name);
        path
    }
}
