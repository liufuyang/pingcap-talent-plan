use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::io::Read;
use std::path::{Path, PathBuf};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

type R<T> = Result<T>;

const MAX_NUM_COMMAND_PER_FILE: usize = 4;

/// The struct to hold key value pairs.
/// Currently it uses memory storage.
pub struct KvStore {
    map: BTreeMap<String, ValueIndex>,

    writer: CursorBufWriter<File>,
    readers: HashMap<usize, BufReader<File>>,
    log_lengths: HashMap<usize, usize>, // keep track of all log file command length. Key is term, value is command length

    term: usize,
    // current term (log file id), start with 1 and continue growing
    num_command: usize,
    // keep track the current writing log file command length
    log_path: PathBuf,
}


struct ValueIndex {
    term: usize,
    head: usize,
    tail: usize,
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
    /// --------------------------------------------------------------------------------------------
    ///
    /// Multi-log-file version notes:
    ///
    /// Keep a value of term: u64 in KvStore to keep track of the current term (start with 1, continue to grow).
    /// Write commands into file under /path/kvs.store/1.log.
    /// And when the number of commands reach MAX_NUM_COMMAND_PER_FILE, increase term by 1, then start writing to
    /// /path/kvs.store/2.log
    ///
    /// When storing the values related to those keys, file positions/offsets are saved as values.
    /// For example:
    ///
    /// (set k1, v1) - (1, 0, 33)
    /// (set k2, v2) - (1, 33, 66)
    /// (rm k1) - (1, 66, 89)
    /// (set k3, v3) - (1, 89, 122)
    ///
    /// (set k4, v4) - (2, 0, 33)
    ///
    /// We keep a number of readers in a readers map to keep a reader for each log file.
    /// We also keep the log file length for each log file in `log_lengths`
    ///
    pub fn open(path: impl Into<PathBuf>) -> R<KvStore> {
        let path = path.into();
        let log_path = path.join("kvs.store");

        // multi file
        let mut map = BTreeMap::new();
        let mut term: usize = 0;
        let mut readers: HashMap<usize, BufReader<File>> = HashMap::new();
        let mut log_lengths: HashMap<usize, usize> = HashMap::new();
        let mut last_log_path: OsString = path.join("kvs.store/1").into_os_string();
        let mut num_command: usize = 0;

        if !log_path.is_dir() {
            create_dir_all(&log_path).expect("log file folder creation failed");
        }

        // check folder empty or not
        let contents: std::fs::ReadDir = log_path.read_dir().expect("read_dir call failed");
        let len = contents.collect::<Vec<_>>().len(); // calculate the amount of items in the directory
        if len != 0 {
            // log file folder not empty, has log files
            term = 0; // set term as 0, to allow comparing with `current_term` below, which is term number read as log file name

            for entry in log_path.read_dir().expect("read_dir call failed") {
                let entry = entry?;

                // TODO delete
                println!("open file: {:?}", &entry.path());

                let current_term: usize = entry.file_name().into_string().expect("log file name into_string failed")
                    .parse().expect("log file name is not int format");
                if !(current_term > term) {
                    panic!("While opening logs, term current is small or equal to term.");
                }


                // open the file firstly for reading to load data on open
                let file = BufReader::new(OpenOptions::new().read(true).open(&entry.path())?);
                let mut stream = Deserializer::from_reader(file).into_iter::<Command>(); // https://docs.serde.rs/serde_json/de/struct.StreamDeserializer.html
                let mut head: usize = 0;
                let mut tail: usize = 0;

                num_command = 0;
                while let Some(command) = stream.next() {
                    tail = stream.byte_offset();

                    if let Ok(command) = command {
                        match command {
                            Command::Set { key, value: _ } => {
                                map.insert(key, ValueIndex { term: current_term, head, tail });
                                num_command += 1;
                            }
                            Command::Remove { key } => {
                                map.remove(key.as_str());
                                num_command += 1;
                            }
                        }
                    }
                    head = tail;
                }
                // finish loading

                // then open again and it save as a it as a value reader
                let reader = BufReader::new(OpenOptions::new().read(true).open(&entry.path())?);
                readers.insert(current_term, reader);
                log_lengths.insert(current_term, num_command);

                // prepare for next loop
                term = current_term;
                last_log_path = entry.path().into_os_string();
            }
        } else {
            // log file folder empty, don't setup reader then
            term = 1;
        }

        // create file if not exist, by creating a writer
        let mut writer = CursorBufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&last_log_path)?,
        )?;

        Ok(KvStore {
            map,
            writer,
            readers,
            log_lengths,
            term,
            num_command,
            log_path,
        })
    }

    fn break_to_new_log_file(&mut self) -> R<()> {
        if self.num_command >= MAX_NUM_COMMAND_PER_FILE {
            self.term += 1;

            let new_log_path = self.log_path.join(self.term.to_string());

            self.writer = CursorBufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&new_log_path)?,
            )?;

            // then open again and it save as a it as a value reader
            let reader = BufReader::new(OpenOptions::new().read(true).open(&new_log_path)?);
            self.readers.insert(self.term, reader);
            self.log_lengths.insert(self.term, 0);
            self.num_command = 0;
        }

        Ok(())
    }

    /// Set key value to store
    pub fn set(&mut self, key: String, value: String) -> R<()> {
        let command = Command::set(key, value);

        // break file if reaching limit
        self.break_to_new_log_file()?;

        let pos_current = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        *self.log_lengths.entry(self.term).or_insert(0) += 1;
        self.num_command += 1;

        match command {
            Command::Set { key, value: _ } => {
                self.map
                    .insert(key, ValueIndex {
                        term: self.term,
                        head: pos_current as usize,
                        tail: self.writer.pos as usize,
                    });
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Get value by a key from store
    pub fn get(&mut self, key: String) -> R<Option<String>> {
        let index = match self.map.get(&key) {
            Some(index) => index,
            None => return Ok(None),
        };

        let mut reader = self.readers.get_mut(&index.term).expect("reader with term x not exist");

        reader.seek(SeekFrom::Start(index.head as u64))?;
        let mut buf = vec![0u8; index.tail - index.head]; // https://stackoverflow.com/questions/30412521/how-to-read-a-specific-number-of-bytes-from-a-stream
        reader.read_exact(&mut buf)?;
        let command: Command = serde_json::from_slice(&buf)?;

        match command {
            Command::Set { key: _, value } => {
                return Ok(Option::Some(value));
            }
            _ => unreachable!(),
        }
    }

    /// Remove key value from store
    pub fn remove(&mut self, key: String) -> R<()> {
        // check key exit:
        if !self.map.contains_key(key.as_str()) {
            return Err(KvsError::NoKeyError);
        }

        // break file if reaching limit
        self.break_to_new_log_file()?;

        let command = Command::remove(key);

        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        // increase log count
        *self.log_lengths.entry(self.term).or_insert(0) += 1;
        self.num_command += 1;

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
//
//impl<W: Write + Seek> Seek for CursorBufWriter<W> {
//    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
//        self.pos = self.writer.seek(pos)?;
//        Ok(self.pos)
//    }
//}
