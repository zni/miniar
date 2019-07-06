use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::str;

const MAGIC_BYTES: [u8; 2] = [0x60, 0x0A];
const SIGNATURE: [u8; 8] = [0x21, 0x3C, 0x61, 0x72, 0x63, 0x68, 0x3E, 0x0A];

pub enum Operation {
    List,
    Unpack
}

pub struct Config {
    pub file: String,
    pub operation: Operation
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 3 {
            return Err("Incorrect number of arguments.");
        }

        let file = args[2].clone();
        let operation = match args[1].as_str() {
            "x"  => Operation::Unpack,
            "ls" => Operation::List,
            _    => return Err("Unknown option"),
        };

        Ok(Config { file, operation })
    }
}

enum ArError {
    EOF,
    IO(std::io::Error),
}

pub struct ArFile {
    pub name: String,
    pub timestamp: String,
    pub owner: String,
    pub group: String,
    pub mode: String,
    pub size: i64,
    pub offset: u64,
}

pub struct Archive {
    pub file: File,
    pub files: Vec<ArFile>,
}

impl Archive {
    pub fn file_listing(&self) {
        let files = self.files.iter();
        for file in files {
            println!("file: {}", file.name);
            println!("timestamp: {}", file.timestamp);
            println!("owner: {}", file.owner);
            println!("group: {}", file.group);
            println!("mode: {}", file.mode);
            println!("size: {}", file.size);
            println!("offset: {}", file.offset);
            println!("");
        }
        println!("");
    }

    pub fn unpack_files(&mut self) -> Result<(), std::io::Error> {
        let files = self.files.iter();
        for file in files {
            self.file.seek(SeekFrom::Start(file.offset))?;

            let mut output_file = File::create(file.name.trim().to_owned())?;
            let mut bytes_read = 0;
            let mut byte = vec![0u8; 1];
            while bytes_read < file.size {
                match self.file.read_exact(byte.as_mut_slice()) {
                    Ok(_) => { bytes_read += 1 },
                    Err(_) => break,
                }

                output_file.write(&byte)?;
            }
        }
        Ok(())
    }

    fn file_size(size: &[u8]) -> i64 {
        let size_string = str::from_utf8(size).unwrap().trim_matches(' ');
        size_string.parse::<i64>().unwrap()
    }

    fn file_header(file : &mut File) -> Result<ArFile, ArError> {
        let mut name = vec![0u8; 16];
        match file.read(name.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        let mut timestamp = vec![0u8; 12];
        match file.read(timestamp.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        let mut owner = vec![0u8; 6];
        match file.read(owner.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        let mut group = vec![0u8; 6];
        match file.read(group.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        let mut mode = vec![0u8; 8];
        match file.read(mode.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        let mut size_buffer = vec![0u8; 10];
        match file.read(size_buffer.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        let mut magic = vec![0u8; 2];
        match file.read(magic.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        assert!(magic.as_slice() == MAGIC_BYTES);

        let size = Archive::file_size(size_buffer.as_slice());
        let pad = size % 2;

        let offset = file.seek(SeekFrom::Current(0)).unwrap();
        if file.seek(SeekFrom::Current(size + pad)).is_err() {
            panic!("malformed archive");
        }

        Ok(ArFile {
            name: String::from_utf8(name).unwrap(),
            timestamp: String::from_utf8(timestamp).unwrap(),
            owner: String::from_utf8(owner).unwrap(),
            group: String::from_utf8(group).unwrap(),
            mode: String::from_utf8(mode).unwrap(),
            size,
            offset,
        })
    }

    fn signature(&mut self) -> Result<bool, ArError> {
        let mut sig_buf: Vec<u8> = vec![0; 8];
        match self.file.read(sig_buf.as_mut_slice()) {
            Ok(0) => return Err(ArError::EOF),
            Ok(_) => (),
            Err(e) => return Err(ArError::IO(e)),
        };

        Ok(sig_buf.as_slice() == SIGNATURE)
    }

    pub fn read_files(&mut self) -> Result<(), &'static str> {
        match self.signature() {
            Ok(false) => return Err("unknown file type"),
            Ok(_) => (),
            Err(_) => return Err("failed to read  signature"),
        }

        while let Ok(arfile) = Archive::file_header(&mut self.file) {
            self.files.push(arfile);
        }

        Ok(())
    }

    pub fn from_path(path: &Path) -> Result<Archive, &'static str> {
        let file = File::open(path).unwrap();
        Ok(Archive { file, files: Vec::new() })
    }
}

pub fn run(config: Config) -> Result<(), &'static str> {
    let path = Path::new(&config.file);
    let mut archive = Archive::from_path(&path)?;
    archive.read_files()?;

    match config.operation {
        Operation::List => archive.file_listing(),
        Operation::Unpack => {
            archive.unpack_files().unwrap_or_else(|_err| {
                println!("failed to unpack files.");
            });
        },
    };
    Ok(())
}
