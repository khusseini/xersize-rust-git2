use super::error::StupidDbError;

use super::models;
use git2::Repository;
use std::path::Path;
static DEFAULT_GIT_SSH_USERNAME: &str = "git";

pub struct StupidRepository {
    path: String,
    pub origin: Option<String>,
    _repo: Option<Repository>,
}

impl StupidRepository {
    pub fn new(path: String) -> StupidRepository {
        StupidRepository {
            path,
            _repo: None,
            origin: None,
        }
    }

    pub fn commit(
        &self,
        tree_oid: git2::Oid,
        blob: &models::StupidBlob,
    ) -> Result<git2::Oid, StupidDbError> {
        if !self._repo.is_some() {
            return Err(StupidDbError::RepoNotInitialized);
        }

        let index_result = self.index();
        if index_result.is_err() {
            return Err(index_result.err().unwrap());
        }

        let repo = self._repo.as_ref().unwrap();
        let signature = repo.signature();
        if signature.is_err() {
            return Err(StupidDbError::SignatureFailed);
        }

        let mut parent_commit = self.find_last_commit();
        if parent_commit.is_none() {
            let result = self.create_initial_commit();
            if result.is_err() {
                return Err(StupidDbError::CommitFetchFailed);
            }

            parent_commit = self.find_last_commit();
            if parent_commit.is_none() {
                return Err(StupidDbError::CommitFetchFailed);
            }
        }

        let tree = self.find_tree(tree_oid);
        if tree.is_none() {
            return Err(StupidDbError::TreeFetchFailed);
        }

        let message = blob.message.as_ref().map_or("no message", |m| m.as_str());

        repo.commit(
            Some("HEAD"),
            signature.as_ref().unwrap(),
            signature.as_ref().unwrap(),
            message,
            tree.as_ref().unwrap(),
            &[parent_commit.as_ref().unwrap()],
        )
        .map_err(|_| StupidDbError::CommitFailed)
    }

    fn find_last_commit(&self) -> Option<git2::Commit> {
        if !self._repo.is_some() {
            return None;
        }

        let head = self._repo.as_ref().unwrap().head();

        if head.is_err() {
            return None;
        }

        let resolved_head = head.unwrap().resolve();

        if resolved_head.is_err() {
            return None;
        }

        let peeled_object = resolved_head.unwrap().peel(git2::ObjectType::Commit);

        if peeled_object.is_err() {
            return None;
        }

        return peeled_object.unwrap().into_commit().ok();
    }

    pub fn find_tree(&self, tree_oid: git2::Oid) -> Option<git2::Tree> {
        if !self._repo.is_some() {
            return None;
        }

        self._repo.as_ref().unwrap().find_tree(tree_oid).ok()
    }

    pub fn index(&self) -> Result<git2::Index, StupidDbError> {
        if !self._repo.is_some() {
            return Err(StupidDbError::RepoNotInitialized);
        }
        self._repo
            .as_ref()
            .unwrap()
            .index()
            .map_err(|_| StupidDbError::IndexFetchFailed)
    }

    pub fn push(&self) -> Result<(), StupidDbError> {
        if !self._repo.is_some() {
            return Err(StupidDbError::RepoNotInitialized);
        }

        let repo = self._repo.as_ref().unwrap();
        let remote_result = repo.find_remote("origin");
        if remote_result.is_err() {
            return Err(StupidDbError::NoOriginRemote);
        }

        let mut remote = remote_result.unwrap();

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, user_from_url, cred_type| {
            let user = user_from_url.unwrap_or(DEFAULT_GIT_SSH_USERNAME);
            if cred_type.contains(git2::CredentialType::USERNAME) {
                git2::Cred::username(user)
            } else {
                git2::Cred::ssh_key_from_agent(user)
            }
        });
        let mut opts = git2::PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote
            .push(&["refs/heads/master"], Some(&mut opts))
            .map_err(|_| StupidDbError::RemotePushFailed)
    }

    pub fn blob(&self, data: &[u8]) -> Result<git2::Oid, StupidDbError> {
        if !self._repo.is_some() {
            return Err(StupidDbError::RepoNotInitialized);
        }
        self._repo
            .as_ref()
            .unwrap()
            .blob(data)
            .map_err(|_| StupidDbError::BlobFailed)
    }

    pub fn open(&mut self) -> Result<&Repository, StupidDbError> {
        match Repository::open(Path::new(&self.path)) {
            Err(_) => Err(StupidDbError::RepoPathInvalid),
            Ok(repo) => {
                let origin = match repo.remotes() {
                    Err(_) => None,
                    Ok(ra) => {
                        if ra.len() > 0 {
                            let remote = String::from(ra.get(0).unwrap());
                            Some(remote)
                        } else {
                            None
                        }
                    }
                };

                if origin.is_some() {
                    self.origin = match repo.find_remote(origin.as_ref().unwrap()) {
                        Err(_) => None,
                        Ok(r) => {
                            let pu = r.url();
                            if pu.is_some() {
                                Some(String::from(r.url().unwrap()))
                            } else {
                                None
                            }
                        }
                    };
                }

                self._repo = Some(repo);
                Ok(self._repo.as_ref().unwrap())
            }
        }
    }

    pub fn init(&self) -> Result<(), StupidDbError> {
        let p = Path::new(self.path.as_str());
        self.check_path(p)
    }

    pub fn set_remoteurl(&self, remote: &str, url: &str) -> Result<(), StupidDbError> {
        if !self._repo.is_some() {
            return Err(StupidDbError::RepoNotInitialized);
        }
        let repo = self._repo.as_ref().unwrap();
        repo.remote_set_url(remote, url)
            .map_err(|_| StupidDbError::RemoteUrlFailed)
    }

    fn check_path(&self, path: &Path) -> Result<(), StupidDbError> {
        if path.exists() {
            return Ok(());
        }

        let mut init_opts = git2::RepositoryInitOptions::new();
        if self.origin.is_some() {
            init_opts.origin_url(self.origin.as_ref().unwrap().as_str());
        }

        match Repository::init_opts(path, &init_opts) {
            Ok(_) => Ok(()),
            Err(e) => Err(StupidDbError::RepoPathInvalid),
        }
    }

    fn create_initial_commit(&self) -> Result<(), StupidDbError> {
        if !self._repo.is_some() {
            return Err(StupidDbError::RepoNotInitialized);
        }

        let index_result = self.index();
        if index_result.is_err() {
            return Err(index_result.err().unwrap());
        }

        let repo = self._repo.as_ref().unwrap();
        let mut index = index_result.unwrap();

        let signature = repo.signature();
        if signature.is_err() {
            return Err(StupidDbError::SignatureFailed);
        }

        let write_result = index.write_tree();
        if write_result.is_err() {
            return Err(StupidDbError::WriteTreeFailed);
        }

        let tree = repo.find_tree(write_result.unwrap());
        if tree.is_err() {
            return Err(StupidDbError::TreeFetchFailed);
        }

        repo.commit(
            Some("HEAD"),
            signature.as_ref().unwrap(),
            signature.as_ref().unwrap(),
            "Initial commit",
            tree.as_ref().unwrap(),
            &[],
        )
        .map_err(|_| StupidDbError::CommitFailed)
        .map(|_| ())
    }
}
