use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref NAME: Regex = Regex::new(r"^[a-zA-Z0-9_-]+\.git$").unwrap();
    pub static ref BLOB_NAME: Regex = Regex::new(r"^(.?[a-zA-Z0-9_-]+)+$").unwrap();
    pub static ref ORIGIN: Regex =
        Regex::new(r"((git|ssh|http(s)?)|(git@[\w\.]+))(:(//)?)([\w\.@:/~-]+)(\.git)(/)?").unwrap();
}
