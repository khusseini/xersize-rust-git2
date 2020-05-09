#[derive(serde::Serialize)]
pub struct StupidRepositoryInfo {
    pub index_length: i32,
    pub commit_length: i64,
}

#[derive(serde::Serialize)]
pub struct StupidDataInfo {
    pub path: String,
    pub id: String,
}
