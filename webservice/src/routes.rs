use diesel::{self, prelude::*};

use rocket_contrib::json::Json;


use crate::schema::*;
use crate::models;
use crate::DbConn;
use rocket::response::status;
use std::fs::{File, create_dir_all};
use std::path::Path;
use reqwest::Url;
use std::process::Command;

/// Route used to check if the api is running
#[get("/")]
pub fn home() -> &'static str {
    "The API is up and running!"
}


/// Route that lists the results available
#[get("/results")]
pub fn list_results(conn: DbConn) -> Json<Vec<models::Result>> {
    let result_list = results::table.load::<models::Result>(&conn.0).expect("Couldn't load results");

    Json(result_list.into_iter().collect::<Vec<models::Result>>())
}

/// Route to add a new result
#[post("/result", format = "json", data = "<request>")]
pub fn add_result(conn: DbConn, request: Json<models::AddResultRequest>) -> status::Accepted<()> {
    let request_result = request.into_inner();

    let save_path = format!("/hostedFiles/results/{}", &request_result.submission_id);
    let dir = Path::new(save_path.as_str());
    create_dir_all(dir);

    let target = request_result.result_url;
    let url = Url::parse(&target).unwrap();

    let file_name = url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    let child = Command::new("bash")
        .arg("-c")
        .arg(format!("wget {} -P {} && cd {} && tar -xvf {}", &target, &save_path, &save_path, &file_name)).spawn()
        .expect("job failed to start");


    let final_result = models::Result {
        id: request_result.submission_id,
        path: String::from(format!("{}/scene_dense_mesh_refine_texture.ply", &request_result.submission_id)),
    };

    diesel::insert_into(results::table)
        .values(final_result)
        .execute(&conn.0);

    status::Accepted::<()>(None)
}