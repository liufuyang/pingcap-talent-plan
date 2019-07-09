use std::collections::{BTreeMap, HashMap};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::Read;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

type R<T> = Result<T>;

/// The struct to hold key value pairs.
/// Currently it uses memory storage.
pub struct KvStore {
    map: BTreeMap<String, (usize, usize)>,

    writer: CursorBufWriter<File>,
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
    /// Create or scan a logfile and create a KvStore from it.
    ///
    /// The this open function will firstly scan through the log file which are concatenated with
    /// multiple JSON elements. And for all the SET entity, store the key to the map; while for all
    /// the REMOVE entity, remove the key from the map.
    ///
    /// When storing the values related to those keys, file positions/offsets are saved as values.
    /// For example:
    ///
    /// (set k1, v1) - (0, 33)
    /// (set k2, v2) - (33, 66)
    /// (rm k1) - (66, 89)
    /// (set k3, v3) - (89, 122)
    ///
    /// KvStore has a writer: CursorBufWriter, which has a filed `pos` is used for keep track of the
    /// current position/cursor of the end of the file.
    ///
    /// After loading the above example, `writer.pos` will be set as 122.
    ///
    /// When adding another (set k4, v4) key-value pair, the value (122, 155) is inserted into index map,
    /// which can be retrieved by k4. And `writer.pos` will be set as 155.
    ///
    pub fn open(path: impl Into<PathBuf>) -> R<KvStore> {
        let path = path.into();
        let file_path = path.join("kvs.store");

        // create file if not exist, by creating a writer
        let mut writer = CursorBufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?,
        )?;
        // create file reader
        let reader = BufReader::new(OpenOptions::new().read(true).open(&file_path)?);

        // open the file again for reading to load data on open
        let file = BufReader::new(OpenOptions::new().read(true).open(&file_path)?);
        let mut map = BTreeMap::new();
        let mut stream = Deserializer::from_reader(file).into_iter::<Command>(); // https://docs.serde.rs/serde_json/de/struct.StreamDeserializer.html

        let mut pos_start: usize = 0;
        let mut pos_end: usize = 0;

        while let Some(command) = stream.next() {
            pos_end = stream.byte_offset();

            if let Ok(command) = command {
                match command {
                    Command::Set { key, value } => {
                        map.insert(key, (pos_start, pos_end));
                    }
                    Command::Remove { key } => {
                        map.remove(key.as_str());
                    }
                }
            }
            pos_start = pos_end;
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
        let pos_current = self.writer.pos;

        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        match command {
            Command::Set { key, value } => {
                self.map
                    .insert(key, (pos_current as usize, self.writer.pos as usize));
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Get value by a key from store
    pub fn get(&mut self, key: String) -> R<Option<String>> {
        let (pos_1, pos_2) = match self.map.get(&key) {
            Some((pos_1, pos_2)) => (*pos_1, *pos_2),
            None => return Ok(None),
        };

        self.reader.seek(SeekFrom::Start(pos_1 as u64))?;
        let mut buf = vec![0u8; pos_2 - pos_1]; // https://stackoverflow.com/questions/30412521/how-to-read-a-specific-number-of-bytes-from-a-stream
        self.reader.read_exact(&mut buf)?;
        let command: Command = serde_json::from_slice(&buf)?;

        match command {
            Command::Set { key, value } => {
                return Ok(Option::Some(value));
            }
            _ => unreachable!(),
        }
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

/// A cursor like BufWriter
struct CursorBufWriter<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64, // keep current file end position
}

impl<W: Write + Seek> CursorBufWriter<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::End(0))?; // keep pos at the end of file. Otherwise do `writer.pos = pos_end as u64;` in function open()

        Ok(CursorBufWriter {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for CursorBufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let offset = self.writer.write(buf)?;
        self.pos += offset as u64;

        Ok(offset)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for CursorBufWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
