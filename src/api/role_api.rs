use crate::{models::role_model::Role, repository::mongodb_repo::MongoRepo};
use rocket::{http::Status, serde::json::Json, State};

#[utoipa::path(
    context_path = "",
    // request_body = Role,
    responses(
        (status = 200, description = "Get role", body = Role)
    )
)]
#[get("/role/<path>")]
pub fn get_role(db: &State<MongoRepo>, path: String) -> Result<Json<Role>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let role = db.get_role_obj(&id);
    match role {
        Ok(role) => Ok(Json(role)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[utoipa::path(
    context_path = "",
    responses(
        (status = 200, description = "Get all roles", body = [Role])
    )
)]
#[get("/roles")]
pub fn get_all_roles(db: &State<MongoRepo>) -> Result<Json<Vec<Role>>, Status> {
    let roles = db.get_all_roles();
    match roles {
        Ok(roles) => Ok(Json(roles)),
        Err(_) => Err(Status::InternalServerError),
    }
}
