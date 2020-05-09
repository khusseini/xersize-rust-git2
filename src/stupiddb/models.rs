use serde::Deserialize;
use serde::Serialize;
use validator::Validate;

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct StupidDbRepositoryConfig {
    #[validate(regex = "crate::stupiddb::regex::NAME")]
    pub name: String,
    #[validate(regex = "crate::stupiddb::regex::ORIGIN")]
    pub origin: Option<String>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct StupidBlob {
    pub id: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub oid: Option<git2::Oid>,
    pub message: Option<String>,
    #[validate(regex = "crate::stupiddb::regex::BLOB_NAME")]
    pub name: String,
    pub content: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct StupidConfig {
    pub root_dir: String,
}
impl ::std::default::Default for StupidConfig {
    fn default() -> Self {
        Self {
            root_dir: "/var/lib/stupiddb/".into(),
        }
    }
}

pub struct StupidEntry {
    pub entry: git2::IndexEntry,
}

impl StupidEntry {
    pub fn new(id: git2::Oid, path: Vec<u8>) -> StupidEntry {
        let entry = git2::IndexEntry {
            id,
            path,
            ctime: git2::IndexTime::new(0, 0),
            mtime: git2::IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: 0o100644,
            uid: 0,
            gid: 0,
            file_size: 0,
            flags: 0,
            flags_extended: 0,
        };

        StupidEntry { entry }
    }
}
