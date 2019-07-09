use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

type R<T> = Result<T>;

/// The struct to hold key value pairs.
/// Currently it uses memory storage.
pub struct KvStore {
    map: HashMap<String, String>,

    writer: FileWriteBuf<File>,
    reader: BufReader<File>,
}

/// A store that keeps key-value pairs in memory
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::open("./");
///
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), Some("value1".to_owned()));
///
/// store.remove("key1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), None);
/// ```
impl KvStore {
    /// Create ro scan a logfile and create a KvStore from it
    pub fn open(path: impl Into<PathBuf>) -> R<KvStore> {
        let path = path.into();
        let file_path = path.join("kvs.store");

        // create file if not exist, by creating a writer
        let writer = FileWriteBuf::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?,
        )?;
        // create file reader
        let reader = BufReader::new(OpenOptions::new().read(true).open(&file_path)?);

        // open the file again for reading to load data on open
        let file = OpenOptions::new().read(true).open(&file_path)?;
        let mut map = HashMap::new();
        let file = BufReader::new(file);
        // https://docs.serde.rs/serde_json/de/struct.StreamDeserializer.html
        let stream = Deserializer::from_reader(file).into_iter::<Command>();
        for command in stream {
            if let Ok(command) = command {
                match command {
                    Command::Set { key, value } => {
                        map.insert(key, value);
                    }
                    Command::Remove { key } => {
                        map.remove(key.as_str());
                    }
                }
            }
        }
        // finish loading

        Ok(KvStore {
            map,
            writer,
            reader,
        })
    }

    /// Set key value to store
    pub fn set(&mut self, key: String, value: String) -> R<()> {
        let command = Command::set(key, value);

        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        match command {
            Command::Set { key, value } => {
                self.map.insert(key, value);
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Get value by a key from store
    pub fn get(&self, key: String) -> R<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }

    /// Remove key value from store
    pub fn remove(&mut self, key: String) -> R<()> {
        // check key exit:
        if !self.map.contains_key(key.as_str()) {
            return Err(KvsError::NO_KEY_ERROR);
        }

        let command = Command::remove(key);

        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        match command {
            Command::Remove { key } => {
                self.map.remove(key.as_str());
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

/// Struct representing a command
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

struct FileWriteBuf<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> FileWriteBuf<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        // TODO
        // println!("new pos: {}", pos);
        Ok(FileWriteBuf {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for FileWriteBuf<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let offset = self.writer.write(buf)?;
        self.pos += offset as u64;
        // TODO
        // println!("pos after write: {}", self.pos);
        Ok(offset)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for FileWriteBuf<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
