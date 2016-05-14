// http://erlang.org/doc/apps/erts/erl_ext_dist.html
use std::io::Read;
use std::io::Result as IoResult;
use std::io::Cursor;
use std::rc::Rc;
use beam::term::Term;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use flate2::read::ZlibDecoder;

// TODO: Support all tag
const TAG_COMPRESSED: u8 = 80;
const TAG_SMALL_INTEGER: u8 = 97;
const TAG_INTEGER: u8 = 98;
const TAG_ATOM: u8 = 100;
const TAG_SMALL_TUPLE: u8 = 104;
const TAG_LARGE_TUPLE: u8 = 105;
const TAG_NIL: u8 = 106;
const TAG_STRING: u8 = 107;
const TAG_LIST: u8 = 108;
const TAG_SMALL_ATOM: u8 = 115;
const TAG_ATOM_UTF8: u8 = 118;
const TAG_SMALL_ATOM_UTF8: u8 = 119;

pub fn from_reader<R: Read>(mut reader: R) -> IoResult<Term> {
    let version = try!(reader.read_u8());
    if version != 131 {
        invalid_data_error(format!("Unknown version: {}", version))
    } else {
        Decoder::new(reader).decode()
    }
}

struct Decoder<R> {
    reader: R,
}
impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader: reader }
    }

    pub fn decode(&mut self) -> IoResult<Term> {
        let tag = try!(self.reader.read_u8());
        match tag {
            TAG_COMPRESSED => self.decode_compressed_term(),
            TAG_SMALL_INTEGER => self.decode_small_integer(),
            TAG_INTEGER => self.decode_integer(),
            TAG_ATOM => self.decode_atom(),
            TAG_SMALL_TUPLE => self.decode_small_tuple(),
            TAG_LARGE_TUPLE => self.decode_large_tuple(),
            TAG_NIL => self.decode_nil(),
            TAG_STRING => self.decode_string(),
            TAG_LIST => self.decode_list(),
            TAG_SMALL_ATOM => self.decode_small_atom(),
            TAG_ATOM_UTF8 => self.decode_atom_utf8(),
            TAG_SMALL_ATOM_UTF8 => self.decode_small_atom_utf8(),
            _ => {
                panic!("Unknown tag: {}", tag);
            }
        }
    }

    fn decode_nil(&mut self) -> IoResult<Term> {
        Ok(Term::new_nil())
    }

    fn decode_integer(&mut self) -> IoResult<Term> {
        let x = try!(self.reader.read_i32::<BigEndian>());
        Ok(Term::new_integer_from_i64(x as i64))
    }

    fn decode_small_integer(&mut self) -> IoResult<Term> {
        let x = try!(self.reader.read_u8());
        Ok(Term::new_integer_from_u64(x as u64))
    }

    fn decode_string(&mut self) -> IoResult<Term> {
        let length = try!(self.reader.read_u16::<BigEndian>());
        let mut chars = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let c = try!(self.reader.read_u8());
            chars.push(Term::new_integer_from_u64(c as u64));
        }
        chars.reverse();

        let mut head = Term::new_nil();
        for c in chars {
            head = Term::new_list(Rc::new(c), Rc::new(head));
        }
        Ok(head)
    }

    fn decode_list(&mut self) -> IoResult<Term> {
        let length = try!(self.reader.read_u32::<BigEndian>());
        let mut elements = Vec::with_capacity(length as usize);
        for _ in 0..length {
            elements.push(try!(self.decode()));
        }
        elements.reverse();

        let mut head = try!(self.decode());
        for e in elements {
            head = Term::new_list(Rc::new(e), Rc::new(head));
        }
        Ok(head)
    }

    fn decode_atom(&mut self) -> IoResult<Term> {
        // TODO: Support latin1 encoded string
        let length = try!(self.reader.read_u16::<BigEndian>());
        let mut name = vec![0; length as usize];
        try!(self.reader.read_exact(&mut name));
        let name = try!(String::from_utf8(name).or_else(|e| invalid_data_error(e.to_string())));
        Ok(Term::new_atom(name))
    }

    fn decode_small_atom(&mut self) -> IoResult<Term> {
        // TODO: Support latin1 encoded string
        let length = try!(self.reader.read_u8());
        let mut name = vec![0; length as usize];
        try!(self.reader.read_exact(&mut name));
        let name = try!(String::from_utf8(name).or_else(|e| invalid_data_error(e.to_string())));
        Ok(Term::new_atom(name))
    }

    fn decode_atom_utf8(&mut self) -> IoResult<Term> {
        let length = try!(self.reader.read_u16::<BigEndian>());
        let mut name = vec![0; length as usize];
        try!(self.reader.read_exact(&mut name));
        let name = try!(String::from_utf8(name).or_else(|e| invalid_data_error(e.to_string())));
        Ok(Term::new_atom(name))
    }

    fn decode_small_atom_utf8(&mut self) -> IoResult<Term> {
        let length = try!(self.reader.read_u8());
        let mut name = vec![0; length as usize];
        try!(self.reader.read_exact(&mut name));
        let name = try!(String::from_utf8(name).or_else(|e| invalid_data_error(e.to_string())));
        Ok(Term::new_atom(name))
    }

    fn decode_small_tuple(&mut self) -> IoResult<Term> {
        let arity = try!(self.reader.read_u8());
        let mut elements = Vec::with_capacity(arity as usize);
        for _ in 0..arity {
            elements.push(Rc::new(try!(self.decode())));
        }
        Ok(Term::new_tuple(elements))
    }

    fn decode_large_tuple(&mut self) -> IoResult<Term> {
        let arity = try!(self.reader.read_u32::<BigEndian>());
        let mut elements = Vec::with_capacity(arity as usize);
        for _ in 0..arity {
            elements.push(Rc::new(try!(self.decode())));
        }
        Ok(Term::new_tuple(elements))
    }

    fn decode_compressed_term(&mut self) -> IoResult<Term> {
        let uncompressed_size = try!(self.reader.read_u32::<BigEndian>());
        let mut buf = Vec::with_capacity(uncompressed_size as usize);
        try!(ZlibDecoder::new(&mut self.reader).read_to_end(&mut buf));
        Decoder::new(Cursor::new(buf)).decode()
    }
}

fn invalid_data_error<T>(message: String) -> IoResult<T> {
    use std::io::Error;
    use std::io::ErrorKind;
    Err(Error::new(ErrorKind::InvalidData, message))
}
