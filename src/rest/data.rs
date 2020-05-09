use rocket_contrib::json::Json;

use crate::stupiddb::error::StupidDbError;
use crate::stupiddb::models;
use crate::stupiddb::StupidDb;

#[post("/<repository>", data = "<blob>")]
pub fn post(
    repository: String,
    blob: models::StupidBlob,
) -> Result<Json<models::StupidBlob>, StupidDbError> {
    let mut db = StupidDb::new();
    let open_error = db.open(&repository).err();
    if open_error.is_some() {
        return Err(open_error.unwrap());
    }

    let hashed_blob = db.hash_object_write(&blob);
    if hashed_blob.is_err() {
        return Err(hashed_blob.err().unwrap());
    }

    let update_error = db.update_index(hashed_blob.as_ref().unwrap()).err();
    if update_error.is_some() {
        return Err(update_error.unwrap());
    }

    let write_tree_result = db.write_tree();
    if write_tree_result.is_err() {
        return Err(write_tree_result.err().unwrap());
    }

    let commit_error = db.commit_tree(write_tree_result.unwrap(), &blob).err();

    if commit_error.is_some() {
        return Err(commit_error.unwrap());
    }

    Ok(Json(hashed_blob.unwrap()))
}
