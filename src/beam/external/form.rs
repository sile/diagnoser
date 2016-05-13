// http://rnyingma.synrc.com/publications/cat/Functional%20Languages/Erlang/BEAM.pdf
// http://www.martinreddy.net/gfx/2d/IFF.txt
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Result as IoResult;
use std::default::Default;
use byteorder::ReadBytesExt;
use byteorder::BigEndian;

#[derive(Debug)]
pub struct Form {
    pub header: Header,
    pub chunks: Vec<Chunk>,
}
impl Form {
    pub fn from_reader<R: Read>(reader: R) -> IoResult<Form> {
        Decoder::new(reader).decode()
    }

    pub fn external_size(&self) -> u32 {
        let initial = self.header.external_size();
        self.chunks.iter().fold(initial, |acc, c| acc + c.external_size())
    }
}

#[derive(Default, Debug)]
pub struct Header {
    pub magic_number: [u8; 4],
    pub form_type: [u8; 4],
}
impl Header {
    pub fn external_size(&self) -> u32 {
        4 + 4 + 4
    }
}

#[derive(Default, Debug)]
pub struct Chunk {
    pub id: [u8; 4],
    pub data: Vec<u8>,
}
impl Chunk {
    pub fn external_size(&self) -> u32 {
        4 + 4 + self.data.len() as u32
    }
}

struct Decoder<R> {
    reader: BufReader<R>,
}
impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader: BufReader::new(reader) }
    }
    pub fn decode(mut self) -> IoResult<Form> {
        let (header, size) = try!(self.decode_header_and_form_size());
        let chunks = try!(self.decode_chunks(size));
        Ok(Form {
            header: header,
            chunks: chunks,
        })
    }
    fn decode_header_and_form_size(&mut self) -> IoResult<(Header, u32)> {
        let mut header = Header::default();
        try!(self.reader.read_exact(&mut header.magic_number));
        let size = try!(self.reader.read_u32::<BigEndian>());
        try!(self.reader.read_exact(&mut header.form_type));
        Ok((header, size - 4))
    }
    fn decode_chunks(&mut self, total_size: u32) -> IoResult<Vec<Chunk>> {
        let mut read_size = 0;
        let mut chunks = Vec::new();
        while read_size < total_size {
            let chunk = try!(self.decode_chunk());
            read_size += chunk.external_size();

            let padding_size = (4 - read_size % 4) % 4;
            self.reader.consume(padding_size as usize);
            read_size += padding_size;

            chunks.push(chunk);
        }
        Ok(chunks)
    }
    fn decode_chunk(&mut self) -> IoResult<Chunk> {
        let mut chunk = Chunk::default();
        try!(self.reader.read_exact(&mut chunk.id));

        let size = try!(self.reader.read_u32::<BigEndian>());
        chunk.data.resize(size as usize, 0);
        try!(self.reader.read_exact(&mut chunk.data));
        Ok(chunk)
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
        let form = Form::from_reader(file).expect("Can't parse file");

        // header
        assert_eq!(b"FOR1", &form.header.magic_number);
        assert_eq!(b"BEAM", &form.header.form_type);

        // chunks
        let expected_list = [(b"Atom", 64),
                             (b"Code", 88),
                             (b"StrT", 0),
                             (b"ImpT", 40),
                             (b"ExpT", 40),
                             (b"LitT", 35),
                             (b"LocT", 4),
                             (b"Attr", 40),
                             (b"CInf", 112),
                             (b"Abst", 238),
                             (b"Line", 22)];
        for (expected, chunk) in expected_list.iter().zip(&form.chunks) {
            assert_eq!(expected.0, &chunk.id);
            assert_eq!(expected.1, chunk.data.len());
        }
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
