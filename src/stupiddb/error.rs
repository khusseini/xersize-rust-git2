#[derive(Debug, Clone)]
pub enum StupidDbError {
    RepoPathInvalid,
    RepoNotInitialized,
    BlobFailed,
    IndexFetchFailed,
    ObjectHashFailed,
    CommitFetchFailed,
    WriteTreeFailed,
    HeadFetchFailed,
    SignatureFailed,
    TreeFetchFailed,
    CommitFailed,
    NoOriginRemote,
    RemoteConnectFailed,
    RemotePushFailed,
    RemoteUrlFailed,
}
