use crate::stupiddb::models::StupidBlob;
use crate::stupiddb::models::StupidDbRepositoryConfig;
use rocket::data::{self, FromDataSimple};
use rocket::http::Status;
use rocket::{Data, Outcome::*, Request};
use std::io::Read;
use validator::Validate;

const LIMIT: u64 = 1 << 20;

impl FromDataSimple for StupidBlob {
    type Error = ValidationError;
    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
        let size_limit = request.limits().get("json").unwrap_or(LIMIT);
        let mut string = String::new().to_owned();
        if let Err(_) = data.open().take(size_limit).read_to_string(&mut string) {
            return Failure((Status::BadRequest, ValidationError::BadFormat));
        }

        let deserialized: Option<Self> = match serde_json::from_str(string.as_str()) {
            Ok(c) => Some(c),
            Err(_) => None,
        };

        if deserialized.is_none() {
            return Failure((Status::BadRequest, ValidationError::BadFormat));
        }

        let t = deserialized.unwrap();
        match t.validate() {
            Ok(_) => Success(t),
            Err(_) => Failure((Status::BadRequest, ValidationError::BadFormat)),
        }
    }
}

impl FromDataSimple for StupidDbRepositoryConfig {
    type Error = ValidationError;
    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
        let size_limit = request.limits().get("json").unwrap_or(LIMIT);
        let mut string = String::new().to_owned();
        if let Err(_) = data.open().take(size_limit).read_to_string(&mut string) {
            return Failure((Status::BadRequest, ValidationError::BadFormat));
        }

        let deserialized: Option<Self> = match serde_json::from_str(string.as_str()) {
            Ok(c) => Some(c),
            Err(_) => None,
        };

        if deserialized.is_none() {
            return Failure((Status::BadRequest, ValidationError::BadFormat));
        }

        let t = deserialized.unwrap();
        match t.validate() {
            Ok(_) => Success(t),
            Err(_) => Failure((Status::BadRequest, ValidationError::BadValue)),
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    BadValue,
    BadFormat,
}
