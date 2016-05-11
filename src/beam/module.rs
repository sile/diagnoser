use std::io::Read;
use std::io::Result as IoResult;

pub struct Module;

impl Module {
    pub fn from_reader<R: Read>(input: R) -> IoResult<Self> {
        external::Deserializer::new(input).deserialize()
    }
}

mod external {
    use std::io::Read;
    use std::io::Result as IoResult;
    use super::Module;

    pub struct Deserializer<R> {
        input: R,
    }
    impl<R: Read> Deserializer<R> {
        pub fn new(input: R) -> Self {
            Deserializer { input: input }
        }
        pub fn deserialize(self) -> IoResult<Module> {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::PathBuf;
    use super::*;

    #[test]
    fn from_reader_works() {
        let file = File::open(test_file("hello.beam")).expect("Can't open file");
        Module::from_reader(file).expect("Can't parse file");
    }

    fn test_file(name: &str) -> PathBuf {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.push("testdata/");
        path.push(name);
        path
    }
}
