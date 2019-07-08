#![doc(html_root_url = "https://liufuyang.github.io/pingcap-talent-plan/")]
#![deny(missing_docs)]

//! A key value store that can store string key
//! and string values onto disk.
//!
//! Also a CLI tool is provided to set and get values.
//!
//! This is a homework project made with the
//! [PingCAP training program](https://github.com/pingcap/talent-plan)

pub use store::KvStore;
pub use error::Result;

mod store;
mod error;
