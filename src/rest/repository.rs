use rocket_contrib::json::Json;

use crate::stupiddb::error::StupidDbError;
use crate::stupiddb::models::StupidDbRepositoryConfig;
use crate::stupiddb::StupidDb;

#[post("/", format = "application/json", data = "<config>")]
pub fn post(
    config: StupidDbRepositoryConfig,
) -> Result<Json<StupidDbRepositoryConfig>, StupidDbError> {
    let mut db = StupidDb::new();
    let result = db.init(&config);
    if result.is_err() {
        return Err(result.err().unwrap());
    }

    let open_result = db.open(&config.name);
    if open_result.is_err() {
        return Err(open_result.err().unwrap());
    }

    let repo = open_result.unwrap();
    let repo_origin = repo.origin.as_ref();
    let config_origin = config.origin.as_ref();

    match (repo_origin, config_origin) {
        (r, Some(co)) => {
            if r.is_none() || !r.unwrap().eq(co) {
                let set_remote_result = repo.set_remoteurl("origin", co.as_str());
                if set_remote_result.is_err() {
                    return Err(set_remote_result.err().unwrap());
                }
            }
        }
        (_, None) => (),
    }

    Ok(Json(config))
}

#[get("/<name>", format = "application/json")]
pub fn get(name: String) -> Result<Json<StupidDbRepositoryConfig>, StupidDbError> {
    let mut db = StupidDb::new();
    db.open(&name).map(|r| {
        let origin = match r.origin.as_ref() {
            Some(o) => Some(String::from(o.as_str())),
            None => None,
        };
        Json(StupidDbRepositoryConfig { name, origin })
    })
}

#[post("/<name>/push")]
pub fn push(name: String) -> rocket::response::status::Accepted<String> {
    let name_copy = name.clone();
    std::thread::spawn(|| {
        let name_in_thread = name_copy;
        let mut db = StupidDb::new();
        let repository = db.open(&name_in_thread);
        if repository.is_err() {
            return;
        }
        match repository.as_ref().unwrap().push() {
            Ok(_) => println!("Repository was pushed. We can notify services."),
            Err(_) => println!("Error occured."),
        }
    });

    rocket::response::status::Accepted(Some(name))
}
