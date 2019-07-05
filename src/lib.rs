use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::str;

const MAGIC_BYTES: [u8; 2] = [0x60, 0x0A];
const SIGNATURE: [u8; 8] = [0x21, 0x3C, 0x61, 0x72, 0x63, 0x68, 0x3E, 0x0A];

pub struct ArFile {
    pub name: String,
    pub timestamp: String,
    pub owner: String,
    pub group: String,
    pub mode: String,
    pub size: i64
}

pub struct Archive {
    pub files: Vec<ArFile>
}

impl Archive {
    fn file_size(size: &[u8]) -> i64 {
        let size_string = str::from_utf8(size).unwrap().trim_matches(' ');
        size_string.parse::<i64>().unwrap()
    }

    fn file_header(file : &mut File) -> Option<ArFile> {
        let mut name = vec![0u8; 16];
        if let Ok(0) = file.read(name.as_mut_slice()) {
            return None
        }

        let mut timestamp = vec![0u8; 12];
        file.read(timestamp.as_mut_slice()).unwrap();

        let mut owner = vec![0u8; 6];
        file.read(owner.as_mut_slice()).unwrap();

        let mut group = vec![0u8; 6];
        file.read(group.as_mut_slice()).unwrap();

        let mut mode = vec![0u8; 8];
        file.read(mode.as_mut_slice()).unwrap();

        let mut size_buffer = vec![0u8; 10];
        file.read(size_buffer.as_mut_slice()).unwrap();

        let mut magic = vec![0u8; 2];
        file.read(magic.as_mut_slice()).unwrap();

        assert!(magic.as_slice() == MAGIC_BYTES);

        let size = Archive::file_size(size_buffer.as_slice());
        let pad = size % 2;

        if file.seek(SeekFrom::Current(size + pad)).is_err() {
            panic!("malformed archive");
        }

        Some(ArFile {
            name: String::from_utf8(name).unwrap(),
            timestamp: String::from_utf8(timestamp).unwrap(),
            owner: String::from_utf8(owner).unwrap(),
            group: String::from_utf8(group).unwrap(),
            mode: String::from_utf8(mode).unwrap(),
            size: size
        })
    }

    fn signature(file: &mut File) -> bool {
        let mut sig_buf: Vec<u8> = vec![0; 8];
        file.read(sig_buf.as_mut_slice()).unwrap();

        sig_buf.as_slice() == SIGNATURE
    }

    fn from_file(file: &mut File) -> Result<Archive, &'static str> {
        let mut files: Vec<ArFile> = Vec::new();

        if !Archive::signature(file) {
            return Err("unknown file type");
        }

        while let Some(arfile) = Archive::file_header(file) {
            files.push(arfile);
        }

        Ok(Archive {
            files: files
        })
    }

    pub fn from_path(path: &Path) -> Result<Archive, &'static str> {
        Archive::from_file(&mut File::open(path).unwrap())
    }
}
