pub mod error;
pub mod info;
pub(super) mod models;
pub(super) mod regex;
pub mod repository;

pub struct StupidDb {
    pub instance_id: String,
    pub root_dir: String,
    repo: Option<repository::StupidRepository>,
}

impl StupidDb {
    pub fn new() -> StupidDb {
        let cfg: models::StupidConfig = confy::load("stupiddb").unwrap();
        let instance_id = uuid::Uuid::new_v4().to_string();
        StupidDb {
            instance_id,
            root_dir: cfg.root_dir,
            repo: None,
        }
    }

    /// Represents git update-index add --cacheinfo oid
    pub fn update_index(&self, blob: &models::StupidBlob) -> Result<(), error::StupidDbError> {
        if !self.repo.is_some() {
            return Err(error::StupidDbError::RepoNotInitialized);
        }

        let index_result = self.repo.as_ref().unwrap().index();

        if index_result.is_err() {
            return Err(index_result.err().unwrap());
        }

        let mut index = index_result.unwrap();
        let index_entry =
            models::StupidEntry::new(blob.oid.unwrap().clone(), blob.name.clone().into_bytes());

        println!("{}", blob.oid.as_ref().unwrap());

        let add_result = index.add(&index_entry.entry).map_err(|e| {
            println!("{}", e);
            return error::StupidDbError::BlobFailed;
        });

        if add_result.is_err() {
            return Err(add_result.err().unwrap());
        }

        index.write().map_err(|e| {
            println!("{}", e);
            return error::StupidDbError::BlobFailed;
        })
    }

    /// Represents git write-tree
    pub fn write_tree(&self) -> Result<git2::Oid, error::StupidDbError> {
        if !self.repo.is_some() {
            return Err(error::StupidDbError::RepoNotInitialized);
        }

        let index_result = self.repo.as_ref().unwrap().index();

        if index_result.is_err() {
            return Err(index_result.err().unwrap());
        }

        let mut index = index_result.unwrap();
        index
            .write_tree()
            .map_err(|_| error::StupidDbError::WriteTreeFailed)
    }

    /// Represents git commit-tree but updates HEAD ref
    pub fn commit_tree(
        &self,
        tree_oid: git2::Oid,
        blob: &models::StupidBlob,
    ) -> Result<git2::Oid, error::StupidDbError> {
        if !self.repo.is_some() {
            return Err(error::StupidDbError::RepoNotInitialized);
        }

        self.repo.as_ref().unwrap().commit(tree_oid, blob)
    }

    /// Represents git hash-object --stdin
    pub fn hash_object(
        &self,
        blob: &models::StupidBlob,
    ) -> Result<models::StupidBlob, error::StupidDbError> {
        if !self.repo.is_some() {
            return Err(error::StupidDbError::RepoNotInitialized);
        }

        let hash_result =
            git2::Oid::hash_object(git2::ObjectType::Blob, blob.content.to_string().as_bytes());

        if hash_result.is_err() {
            return Err(error::StupidDbError::ObjectHashFailed);
        }

        let mut new_blob = blob.clone();
        new_blob.oid = Some(hash_result.unwrap());
        new_blob.id = Some(new_blob.oid.unwrap().to_string());
        Ok(new_blob)
    }

    /// Represents git hash-object --stdin -w
    pub fn hash_object_write(
        &self,
        blob: &models::StupidBlob,
    ) -> Result<models::StupidBlob, error::StupidDbError> {
        let result = self.hash_object(blob);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let repo = self.repo.as_ref().unwrap();
        let blob_result = repo.blob(blob.content.to_string().as_bytes());

        if blob_result.is_err() {
            return Err(error::StupidDbError::BlobFailed);
        }

        let mut new_blob = blob.clone();
        new_blob.oid = Some(blob_result.unwrap());
        new_blob.id = Some(new_blob.oid.unwrap().to_string());
        Ok(new_blob)
    }

    /// Opens a previously created repository
    pub fn open(
        &mut self,
        name: &String,
    ) -> Result<&repository::StupidRepository, error::StupidDbError> {
        if self.repo.is_some() {
            return Ok(self.repo.as_ref().unwrap());
        }

        let path = self.get_repo_full_path(name);
        let mut stupidrepo = repository::StupidRepository::new(path);

        let res = stupidrepo.open();
        if res.is_ok() {
            self.repo = Some(stupidrepo);
            return Ok(self.repo.as_ref().unwrap());
        }

        return Err(res.err().unwrap());
    }

    /// Represents git init but with origin if given
    pub fn init(
        &mut self,
        config: &models::StupidDbRepositoryConfig,
    ) -> Result<(), error::StupidDbError> {
        let path = self.get_repo_full_path(&config.name);
        let mut stupidrepo = repository::StupidRepository::new(path);
        if config.origin.is_some() {
            stupidrepo.origin = Some(String::from(config.origin.as_ref().unwrap().as_str()));
        }

        if stupidrepo.open().is_err() {
            let result = stupidrepo.init();
            if result.is_err() {
                return result;
            }
        }

        self.repo = Some(stupidrepo);
        Ok(())
    }

    fn get_repo_full_path(&self, name: &String) -> String {
        let mut s = self.root_dir.to_string();
        s.push_str(name.as_str());
        s
    }
}
