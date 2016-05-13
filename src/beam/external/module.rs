use std;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Cursor;
use std::collections::HashSet;
use beam::term::Term;
use beam::term::Atom;
use beam::term::ExternalFun;
use beam::term::Arity;
use beam::external::form;
use beam::external::form::Form;
use byteorder::ReadBytesExt;
use byteorder::BigEndian;

#[derive(Default)]
pub struct Module {
    pub atoms: Option<Vec<Atom>>,
    pub imports: Option<Vec<ExternalFun>>,
    pub exports: Option<Vec<Export>>,
    pub abstract_form: Option<Term>,
    pub unknown_chunks: Vec<form::Chunk>,
}

pub struct Export {
    pub function: Atom,
    pub arity: Arity,
    pub label: CodePosition,
}
impl Export {
    pub fn new(function: Atom, arity: Arity, label: CodePosition) -> Self {
        Export {
            function: function,
            arity: arity,
            label: label,
        }
    }
    pub fn to_tuple(&self) -> (String, Arity, CodePosition) {
        (self.function.to_string(), self.arity, self.label)
    }
}

pub type CodePosition = usize;

impl Module {
    pub fn from_reader<R: Read>(reader: R) -> IoResult<Module> {
        let form = try!(Form::from_reader(reader));
        try!(validate_form_header(&form.header));

        let mut knowns = HashSet::new();
        let mut module = Module::default();
        for chunk in form.chunks {
            if knowns.contains(&chunk.id) {
                return invalid_data_error(format!("Duplicated '{}' chunk",
                                                  std::str::from_utf8(&chunk.id)
                                                      .unwrap_or(&format!("{:?}", chunk.id))));
            }
            knowns.insert(chunk.id.clone());
            match &chunk.id {
                b"Atom" => try!(module.load_atoms(Cursor::new(chunk.data))),
                b"ImpT" => try!(module.load_imports(Cursor::new(chunk.data))),
                b"ExpT" => try!(module.load_exports(Cursor::new(chunk.data))),
                b"Abst" => try!(module.load_abstract_form(Cursor::new(chunk.data))),
                _ => module.unknown_chunks.push(chunk),
            }
        }
        Ok(module)
    }

    fn load_atoms<R: Read>(&mut self, mut reader: R) -> IoResult<()> {
        let count = try!(reader.read_u32::<BigEndian>()) as usize;
        let mut atoms = Vec::with_capacity(count);
        let mut buf = [0; 0x100];
        for _ in 0..count {
            let length = try!(reader.read_u8()) as usize;
            try!(reader.read_exact(&mut buf[0..length]));
            let value = try!(std::str::from_utf8(&buf[0..length])
                                 .or_else(|e| invalid_data_error(e.to_string())))
                            .to_string();
            atoms.push(Atom::new(value));
        }
        self.atoms = Some(atoms);
        Ok(())
    }

    fn load_imports<R: Read>(&mut self, mut reader: R) -> IoResult<()> {
        let count = try!(reader.read_u32::<BigEndian>()) as usize;
        let mut imports = Vec::with_capacity(count);
        for _ in 0..count {
            let module_id = try!(reader.read_u32::<BigEndian>()) - 1;
            let function_id = try!(reader.read_u32::<BigEndian>()) - 1;
            let arity = try!(reader.read_u32::<BigEndian>());
            let import = ExternalFun {
                module: try!(self.get_atom(module_id as usize)),
                function: try!(self.get_atom(function_id as usize)),
                arity: arity as Arity,
            };
            imports.push(import);
        }
        self.imports = Some(imports);
        Ok(())
    }

    fn load_exports<R: Read>(&mut self, mut reader: R) -> IoResult<()> {
        let count = try!(reader.read_u32::<BigEndian>()) as usize;
        let mut exports = Vec::with_capacity(count);
        for _ in 0..count {
            let function_id = try!(reader.read_u32::<BigEndian>()) - 1;
            let arity = try!(reader.read_u32::<BigEndian>());
            let label = try!(reader.read_u32::<BigEndian>());
            let export = Export {
                function: try!(self.get_atom(function_id as usize)),
                arity: arity as Arity,
                label: label as CodePosition,
            };
            exports.push(export);
        }
        self.exports = Some(exports);
        Ok(())
    }

    fn load_abstract_form<R: Read>(&mut self, reader: R) -> IoResult<()> {
        let abstract_form = try!(super::term::from_reader(reader));
        self.abstract_form = Some(abstract_form);
        Ok(())
    }

    fn get_atom(&self, atom_id: usize) -> IoResult<Atom> {
        if let Some(ref atoms) = self.atoms {
            if atom_id >= atoms.len() {
                invalid_data_error(format!("Too large atom id: {} (max={})",
                                           atom_id,
                                           atoms.len() - 1))
            } else {
                Ok(atoms[atom_id].clone())
            }
        } else {
            invalid_data_error("Missing 'Atom' preceding chunk".to_string())
        }
    }
}

fn validate_form_header(header: &form::Header) -> IoResult<()> {
    if b"FOR1" != &header.magic_number {
        invalid_data_error(format!("Unexpected magic number: {:?}", header.magic_number))
    } else if b"BEAM" != &header.form_type {
        invalid_data_error(format!("Unexpected form type: {:?}", header.form_type))
    } else {
        Ok(())
    }
}

fn invalid_data_error<T>(message: String) -> IoResult<T> {
    Err(IoError::new(ErrorKind::InvalidData, message))
}

#[cfg(test)]
mod tests {
    use std;
    use std::fs::File;
    use std::path::PathBuf;
    use beam::term::*;
    use super::*;

    #[test]
    fn from_reader_works() {
        let file = File::open(test_file("hello.beam")).expect("Can't open file");
        let module = Module::from_reader(file).expect("Can't parse file");

        // Atom chunk
        assert_eq!(vec!["hello".to_string(),
                        "world".to_string(),
                        "io".to_string(),
                        "format".to_string(),
                        "ok".to_string(),
                        "module_info".to_string(),
                        "erlang".to_string(),
                        "get_module_info".to_string()],
                   module.atoms.unwrap().iter().map(|a| a.to_string()).collect::<Vec<_>>());

        // ImpT chunk
        assert_eq!(vec!["fun io:format/1".to_string(),
                        "fun erlang:get_module_info/1".to_string(),
                        "fun erlang:get_module_info/2".to_string()],
                   module.imports.unwrap().iter().map(|x| x.to_string()).collect::<Vec<_>>());

        // ExpT chunk
        assert_eq!(vec![("module_info".to_string(), 1, 6),
                        ("module_info".to_string(), 0, 4),
                        ("world".to_string(), 0, 2)],
                   module.exports.unwrap().iter().map(|x| x.to_tuple()).collect::<Vec<_>>());

        // Abst chunk
        assert_eq!(Term::Atom(Atom::new("TODO".to_string())),
                   module.abstract_form.unwrap());

        // Remaining chunks
        assert_eq!(vec!["Code".to_string(),
                        "StrT".to_string(),
                        "LitT".to_string(),
                        "LocT".to_string(),
                        "Attr".to_string(),
                        "CInf".to_string(),
                        "Line".to_string()],
                   module.unknown_chunks
                         .iter()
                         .map(|c| std::str::from_utf8(&c.id).unwrap().to_string())
                         .collect::<Vec<_>>());
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
