use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::{create_dir_all, DirEntry, File, OpenOptions, remove_file};
use std::io;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::io::Read;
use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::engines::KvsEngine;
use crate::engines::counter::LengthCount;
use crate::error::{KvsError, Result};

type R<T> = Result<T>;

const MAX_NUM_COMMAND_PER_FILE: usize = 1024 * 10;
const COMPACTION_THRESHOLD: f64 = 0.618;

/// The struct to hold key value pairs.
/// Currently it uses memory storage.
pub struct KvStore {
    /// index map, key as store String key, value as indexes to find the actual String value
    map: BTreeMap<String, ValueIndex>,

    writer: CursorBufWriter<File>,
    readers: HashMap<usize, BufReader<File>>,

    /// current term (log file id), start with 1 and continue growing
    term: usize,

    /// keep track of all log file command length. Key is term, value is command length
    log_lengths: HashMap<usize, LengthCount>,

    /// keep track the current writing log file command length
    current_log_len: usize,

    /// keep track of the current dir for saving log files
    log_path: PathBuf,

}


struct ValueIndex {
    term: usize,
    head: usize,
    tail: usize,
}

/// # KvStore : A simple Log-structured key value store
///
/// ## Examples:
///
/// This is am example how you can use this KvStore:
/// ```rust
/// # use kvs::{KvStore, KvsEngine};
/// let mut store = KvStore::open("./").unwrap();
///
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(store.get("key1".to_owned()).unwrap(), Some("value1".to_owned()));
///
/// store.remove("key1".to_owned());
/// assert_eq!(store.get("key1".to_owned()).unwrap(), None);
/// ```
///
/// When storing the values related to those keys, the file positions/offsets are saved as values in an
/// `index map` in memory.
///
/// For example, conceptually key and value positions are stored in memroy:
/// ```text
/// (set k1, v1) -> k1: (0, 33)
/// (set k2, v2) -> k2: (33, 66)
/// (rm k1)      -> k3: (66, 89)
/// (set k3, v3) -> k4: (89, 122)
/// ```
///
/// Actual key value pairs (commands) are saved in file. For example,
/// a log file would look something like:
/// ```json
/// {"Set":{"key":"k1","value":"v1"}}{"Remove":{"key":"k1"}}{"Set":{"key":"k1","value":"v1"}}{"Set":{"key":"k2","value":"v2"}}
/// ```
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
/// ## Multi-log-file version notes:
///
/// Keep a value of term: u64 in KvStore to keep track of the current term (start with 1, continue to grow).
/// Write commands into file under /path/kvs.store/1.log.
/// And when the number of commands reach MAX_NUM_COMMAND_PER_FILE, increase term by 1, then start writing to
/// /path/kvs.store/2.log
///
/// When storing the values related to those keys, file the term number and positions/offsets are saved as values.
/// For example:
/// ```text
/// (set k1, v1) -> k1: (1, 0, 33)
/// (set k2, v2) -> k2: (1, 33, 66)
/// (rm k1)      -> k3: (1, 66, 89)
/// (set k3, v3) -> k4: (1, 89, 122)
///
/// (set k4, v4) -> (2, 0, 33)  # this writes into a new file
/// ```
/// We keep a number of readers in a readers map to keep a reader for each log file.
/// We also keep the log file length for each log file in `log_lengths`
///
///
impl KvStore {
    /// Create or scan a logfile and create a KvStore from it.
    ///
    /// The this open function will firstly scan through the log file which are concatenated with
    /// multiple JSON elements.
    /// * And for all the SET command, store the key to the index map
    /// * while for all the REMOVE command, remove the key from the index map
    ///
    /// At the same time, log_lengths - a map keeps track of all log file command length is also
    /// created in memory.
    ///
    /// Also a reader for each term file is created, and a writer is created for the last term file
    /// to append on.
    ///
    pub fn open(path: impl Into<PathBuf>) -> R<KvStore> {
        let path = path.into();
        let log_path = path.join("kvs.store");
        create_dir_all(&log_path).expect("log file folder creation failed");

        // multi file
        let mut map: BTreeMap<String, ValueIndex> = BTreeMap::new();
        let mut term: usize;
        let mut readers: HashMap<usize, BufReader<File>> = HashMap::new();
        let mut log_lengths: HashMap<usize, LengthCount> = HashMap::new();
        let mut last_log_path: OsString = path.join("kvs.store/1").into_os_string();
        let mut current_log_len: usize = 0;

        // check folder empty or not
        let contents: std::fs::ReadDir = log_path.read_dir().expect("read_dir call failed");
        let log_file_count = contents.collect::<Vec<_>>().len(); // calculate the amount of items in the directory
        if log_file_count != 0 {
            // log file folder not empty, has log files
            term = 0; // set term as 0, to allow comparing with `current_term` below, which is term number read as log file name

            // sort log files
            let logs = log_path.read_dir().expect("read_dir call failed").into_iter()
                .filter(|f| dir_entry_to_usize(f.as_ref().unwrap()).is_ok())
                .sorted_by(|a, b| {
                    let a = &dir_entry_to_usize(a.as_ref().unwrap()).expect("log file name is not int format");
                    let b = &dir_entry_to_usize(b.as_ref().unwrap()).expect("log file name is not int format");
                    Ord::cmp(a, b)
                });
            for entry in logs {
                let entry = entry?;

                let current_term: usize = entry.file_name().into_string().expect("log file name into_string failed")
                    .parse().expect("log file name is not int format");
                if !(current_term > term) {
                    panic!("While opening logs, term current is small or equal to term.");
                }

                // open the file firstly for reading to load data on open
                let file = BufReader::new(OpenOptions::new().read(true).open(&entry.path())?);
                let mut stream = Deserializer::from_reader(file).into_iter::<Command>(); // https://docs.serde.rs/serde_json/de/struct.StreamDeserializer.html
                let mut head: usize = 0;
                let mut tail: usize;

                let mut current_log_len_count = LengthCount::new();

                current_log_len = 0;

                while let Some(command) = stream.next() {
                    tail = stream.byte_offset();

                    if let Ok(command) = command {
                        match command {
                            Command::Set { key, value: _ } => {

                                // if the key already set before, then garbage exist
                                if let Some(old_index) =  map.get(&key) {
                                    if old_index.term == current_term { // garbage at current term
                                        current_log_len_count.increase_len_with_garbage();
                                    } else { // garbage at previous term
                                        let old_log_len_count = log_lengths.get_mut(&old_index.term).expect("log_length has no term key");
                                        old_log_len_count.increase_garbage_len();
                                        current_log_len_count.increase_len();
                                    }
                                } else { // a new set key
                                    current_log_len_count.increase_len();
                                }

                                map.insert(key, ValueIndex { term: current_term, head, tail });
                                current_log_len += 1;
                            }
                            Command::Remove { key } => {

                                // if the key already set before (here should always be true), then garbage exist
                                if let Some(old_index) =  map.get(&key) {
                                    if old_index.term == current_term { // garbage at current term
                                        current_log_len_count.increase_garbage_len(); // count the set command as garbage
                                        current_log_len_count.increase_len_with_garbage(); // increase length and count the remove command is also garbage
                                    } else { // garbage at previous term
                                        let old_log_len_count = log_lengths.get_mut(&old_index.term).expect("log_length has no term key");
                                        old_log_len_count.increase_garbage_len();
                                        current_log_len_count.increase_len_with_garbage();
                                    }
                                } else {
                                    println!("Warning: on opening, a Remove command encounter but without any previous set. Neglect it and moving on.");
                                }

                                map.remove(key.as_str());
                                current_log_len += 1;
                            }
                        }
                    }
                    head = tail;
                }
                // finish loading

                // then open again and it save as a it as a value reader
                let reader = BufReader::new(OpenOptions::new().read(true).open(&entry.path())?);
                readers.insert(current_term, reader);
                log_lengths.insert(current_term, current_log_len_count);

                // prepare for next loop
                term = current_term;
                last_log_path = entry.path().into_os_string();
            }
        } else {
            // log file folder empty, do nothing but set term as init value 1
            term = 1;
        }

        // Create writer. Also create log file to write if not exist, by creating this writer
        let writer = CursorBufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&last_log_path)?,
        )?;

        // Create reader again when no log files found, otherwise readers will already be created above.
        if log_file_count == 0 {
            let reader = BufReader::new(OpenOptions::new().read(true).open(&last_log_path)?);
            readers.insert(term, reader);
            log_lengths.insert(term, LengthCount::new());
        }

        Ok(KvStore {
            map,
            writer,
            readers,
            term,
            log_lengths,
            current_log_len,
            log_path,
        })
    }
//
//    fn set_temp_dir(&mut self, temp_dir: TempDir) {
//        self.tmp_dir = temp_dir;
//    }


    fn break_to_new_log_file(&mut self) -> R<()> {

        self.term += 1;

        let new_log_path = self.log_path.join(self.term.to_string());

        // TODO: Here may fail, the dir will be removed by temp dir if nothing holds it in the scope KvStore is in
        // TODO: Create a better error message
        // create_dir_all(&self.log_path).expect("log file folder creation failed");

        let new_file = OpenOptions::new()
        .create(true)
            .write(true)
            .append(true)
            .open(&new_log_path).expect("break_to_new_log_file(): log file creation failed. Check whether temp folder got cleaned up while store exist");

        self.writer = CursorBufWriter::new(new_file)?;

        // then open again and it save as a it as a value reader
        let reader = BufReader::new(OpenOptions::new().read(true).open(&new_log_path)?);
        self.readers.insert(self.term, reader);
        self.log_lengths.insert(self.term, LengthCount::new());
        self.current_log_len = 0;

        Ok(())
    }

    /// Compaction
    ///
    /// This function is called when we know a log file of certain term has it's
    /// garbage rate is larger than the compaction threshold. We already calculated the
    /// garbage rate when self.set(key, value) or self.remove(key) function is called,
    /// specifically when we know the garbage is at a previous term (we know as we compare the
    /// key's index's term is not the current term.)
    ///
    /// Compaction is done by going through the term file to compact, finding all the Set Command
    /// that is still effective, then write these commands at the end of the current term file.
    /// During the process we update the index map, remove and consume the reader of the compaction term,
    /// update log_lengths map, then finally remove the term file.
    ///
    fn compaction(&mut self, term: usize) -> R<()> {
        // check whether compaction happening on the same file
        // if so, and when only when self.current_log_len < MAX_NUM_COMMAND_PER_FILE
        // (meaning break_to_new_log_file() won't be called immediately when self.set(..) is called)
        // we make a new term and file to write
        if term == self.term && self.current_log_len < MAX_NUM_COMMAND_PER_FILE{
            self.break_to_new_log_file()?;
        }

        let mut reader = self.readers.remove(&term).expect("Get old reader failed");
        reader.seek(SeekFrom::Start(0))?;

        let mut temp_map: HashMap<String, String> = HashMap::new();

        let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
        while let Some(command) = stream.next() {
            if let Ok(command) = command {
                match command {
                    Command::Set {key, value} => {
                        if let Some(index) = self.map.get(&key) {
                            if index.term == term { // meaning this key value pair is still valid and stored in this term
                                temp_map.insert(key, value);
                            }
                        }
                    },
                    _ => (),
                }
            }
        }

        let effective_element_len = self.log_lengths.get(&term).expect("log_lengths has no term").effective_len();
        let temp_map_len = temp_map.len();
        if effective_element_len != temp_map_len {
            panic!(format!("Compaction bug: effective element number {} is different from temp_map len {}", effective_element_len, temp_map_len));
        }

        // TODO - delete
        // println!("Garbage collect on term: {}, writing {} previous active commands.", term, effective_element_len);

        for (k, v) in temp_map.into_iter() {
            self.map.remove(&k).expect("Compaction error - remove key from index map");
            self.set(k, v)?;
        }
        self.log_lengths.remove(&term).expect("Compaction error - remove term from log_lengths");
        // finally delete the file
        remove_file(self.log_path.join(term.to_string()))?;

        Ok(())
    }
}


impl KvsEngine for KvStore {
    /// Get value by a key from store
    fn get(&mut self, key: String) -> R<Option<String>> {
        let index = match self.map.get(&key) {
            Some(index) => index,
            None => return Ok(None),
        };

        let reader = self.readers.get_mut(&index.term).expect(&format!("reader with term {} not exist", &index.term));
        reader.seek(SeekFrom::Start(index.head as u64))?;
        let mut buf = vec![0u8; index.tail - index.head]; // https://stackoverflow.com/questions/30412521/how-to-read-a-specific-number-of-bytes-from-a-stream
        reader.read_exact(&mut buf)?;
        let command: Command = serde_json::from_slice(&buf)?;

        // TODO: delete
        // println!("log_lengths: {:?}", self.log_lengths);

        match command {
            Command::Set { key: _, value } => {
                return Ok(Option::Some(value));
            }
            _ => unreachable!(),
        }
    }


    /// Set key value to store
    ///
    /// Operation include:
    /// * write command to file
    /// * update log_lengths map
    /// * update current_log_len
    /// * update index map
    fn set(&mut self, key: String, value: String) -> R<()> {
        // break file if reaching limit
        if self.current_log_len >= MAX_NUM_COMMAND_PER_FILE {
            self.break_to_new_log_file()?;
        }

        let command = Command::set(key, value);
        let pos_current = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        let key = match command { // own String key again
            Command::Set{ key, value: _} => key,
            _ => unreachable!()
        };

        // increase log count
        // if the key already set before, then garbage exist
        let mut compaction_term: usize = 0;
        if let Some(old_index) = self.map.get(&key) {
            if old_index.term == self.term { // garbage at current term
                let current_log_len_count = self.log_lengths.get_mut(&self.term).expect("log_length has no term key");
                current_log_len_count.increase_len_with_garbage();

                if current_log_len_count.garbage_rate() > COMPACTION_THRESHOLD {
                    compaction_term = self.term;
                }
            } else { // garbage at previous term
                let old_log_len_count = self.log_lengths.get_mut(&old_index.term).expect("log_length has no term key");
                old_log_len_count.increase_garbage_len();

                if old_log_len_count.garbage_rate() > COMPACTION_THRESHOLD {
                    compaction_term = old_index.term;
                }

                let current_log_len_count = self.log_lengths.get_mut(&self.term).expect("log_length has no term key");
                current_log_len_count.increase_len();
            }
        } else { // this is a new key
            let current_log_len_count = self.log_lengths.entry(self.term).or_insert(LengthCount::new());
            current_log_len_count.increase_len();
        }

        self.current_log_len += 1;

        self.map
            .insert(key, ValueIndex {
                term: self.term,
                head: pos_current as usize,
                tail: self.writer.pos as usize,
            });


        // TODO: delete
        // println!("log_lengths: {:?}", self.log_lengths);

        if compaction_term > 0  {
            self.compaction(compaction_term)?;
        }

        Ok(())
    }

    /// Remove key value from store
    ///
    /// Operation include:
    /// * write command to file
    /// * update log_lengths map
    /// * update current_log_len
    /// * update index map
    fn remove(&mut self, key: String) -> R<()> {
        // check key exit:
        if !self.map.contains_key(key.as_str()) {
            return Err(KvsError::KeyNotFound);
        }

        // break file if reaching limit
        if self.current_log_len >= MAX_NUM_COMMAND_PER_FILE {
            self.break_to_new_log_file()?;
        }

        let command = Command::remove(key);
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        let key = match command { // own String key again
            Command::Remove{ key} => key,
            _ => unreachable!()
        };

        // increase log count
        // if the key already set before (here should always be true), then garbage exist
        let mut compaction_term: usize = 0;
        if let Some(old_index) = self.map.get(&key) {
            if old_index.term == self.term { // garbage at current term
                let current_log_len_count = self.log_lengths.get_mut(&self.term).expect("log_length has no term key");
                current_log_len_count.increase_garbage_len(); // count the set command as garbage
                current_log_len_count.increase_len_with_garbage(); // increase length and count the remove command is also garbage

                if current_log_len_count.garbage_rate() > COMPACTION_THRESHOLD {
                    compaction_term = self.term;
                }
            } else { // garbage at previous term
                let old_log_len_count = self.log_lengths.get_mut(&old_index.term).expect("log_length has no term key");
                old_log_len_count.increase_garbage_len();
                if old_log_len_count.garbage_rate() > COMPACTION_THRESHOLD {
                    compaction_term = old_index.term;
                }
                let current_log_len_count = self.log_lengths.get_mut(&self.term).expect("log_length has no term key");
                current_log_len_count.increase_len_with_garbage();
            }
        } else {
            unreachable!();
        }

        self.current_log_len += 1;

        self.map.remove(key.as_str());


        // TODO: delete
        // println!("log_lengths: {:?}", self.log_lengths);

        if compaction_term > 0 {
            self.compaction(compaction_term)?;
        }

        Ok(())
    }
}

fn dir_entry_to_usize(entry: &DirEntry) -> R<usize> {
    entry.file_name().into_string().expect("log file name into_string failed")
        .parse().map_err(KvsError::ParseIntError)
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
