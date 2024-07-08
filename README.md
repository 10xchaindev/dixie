# Dixie 

Dixie is a web framework built in rocket and mongodb. It's meant to make building a CRUD api faster on rust.

## Example 

### Initialize 
```rs
use dotenv::dotenv;

use rocket::{fairing::AdHoc, routes};
use rocket_db_pools::Database;

use modules::{auth::auth_route::auth_route, course::course_route};
use dixie::{config::Config, db::Db};

pub mod modules;

#[rocket::main]
async fn main() {
    dotenv().ok();

    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::config::<Config>())
        .mount("/auth", routes![auth_route::authenticate])
        .mount(
            "/courses",
            routes![
                course_route::get_courses,
                course_route::get_course,
                course_route::get_user_courses,
                course_route::create_user_course,
                course_route::update_user_course,
            ],
        )
        .launch()
        .await
        .unwrap();
}
```

### Models 

To create a model implement dixie::model::Model 


```rs

use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use dixie::core::db::Model;

pub static DB_NAME: &str = "courses";

#[derive(Serialize, Deserialize)]
pub struct Course {
    title: String,
    description: String,
    artwork: String,
}

impl Model for Course {
    fn collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(&DB_NAME)
    }
}
```

### Modules 
For creating read controller. impl the `dixie::Write` trait
For creating write controller. impl the `dixie::Read` trait

Then use helper function from those trait for more complex query or not

```rs
use std::{error::Error, str::FromStr};

use mongodb::bson::{doc, oid::ObjectId};
use rocket::{http::Status, serde::json::Json};

use dixie::{
    core::{
        crud::{read::Read, write::Write},
        header_map::context::Context,
        pagination::limit_offset::{LimitOffsetPagination, Query},
    },
    models::course::Course,
};

pub struct CourseController;

impl Read<Course> for CourseController {}
impl Write<Course> for CourseController {}

impl CourseController {
    pub async fn get_courses<'a>(
        context: Context<'a>,
        query: Query,
    ) -> Result<Json<LimitOffsetPagination<Course>>, (Status, mongodb::error::Error)> {
        match CourseController::get_all(&context, &query, None).await {
            Ok(courses) => Ok(Json(courses)),
            Err(error) => Err(error),
        }
    }

    pub async fn get_course<'a>(
        context: Context<'a>,
        id: &str,
    ) -> Result<Json<Course>, (Status, Box<dyn Error>)> {
        let filter = doc! {id: ObjectId::from_str(id).unwrap()};

        match CourseController::get_one(&context, filter).await {
            Ok(course) => Ok(Json(course)),
            Err(error) => Err(error),
        }
    }
}
```


### Routers

Then use rocket convention to create routes

```rs
use mongodb::bson::doc;
use mongodb::options::UpdateModifications;
use rocket::{get, http::Status, patch, post, serde::json::Json, State};

use crate::core::header_map::context::Context;

use crate::{
    core::{
        config::Config,
        db::Rel,
        header_map::authorization::Authorization,
        pagination::limit_offset::{LimitOffsetPagination, Query},
    },
    models::{
        course::Course,
        user_course::{UserCourse, UserCourseRel},
    },
};

use super::course_controller;
use super::user_course_controller::UserCourseController;

#[get("/?<query..>", format = "json")]
pub async fn get_courses<'a>(
    context: Context<'a>,
    query: Option<Query>,
    config: &State<Config>,
) -> Result<Json<LimitOffsetPagination<Course>>, (Status, ())> {
    let query = Query::default(config, query);

    Ok(
        course_controller::CourseController::get_courses(context, query)
            .await
            .unwrap(),
    )
}

#[get("/<id>", format = "json")]
pub async fn get_course<'a>(context: Context<'a>, id: &str) -> Result<Json<Course>, (Status, ())> {
    let result = course_controller::CourseController::get_course(context, id).await;

    match result {
        Ok(course) => Ok(course),
        Err((status, error)) => Err((status, ())),
    }
}

#[get("/me", format = "json")]
pub async fn get_user_courses<'a>(
    context: Context<'a>,
    authorization: Authorization,
) -> Json<Vec<UserCourseRel>> {
    let Authorization(user) = authorization;

    let filter = doc! {"user_id": user.id};

    let user_courses = UserCourseRel::find(&context.db, filter).await.unwrap();

    Json(user_courses)
}

#[get("/me/<id>", format = "json")]
pub async fn get_user_course<'a>(
    context: Context<'a>,
    authorization: Authorization,
    id: &str,
) -> Json<UserCourseRel> {
    let Authorization(user) = authorization;

    let filter = doc! {"_id": id, "user_id": user.id};
    let user_course = UserCourseRel::find_one(&context.db, filter).await.unwrap();
    Json(user_course)
}

#[post("/me", format = "json", data = "<data>")]
pub async fn create_user_course<'a>(
    context: Context<'a>,
    authorization: Authorization,
    data: Json<UserCourse>,
) -> Result<Json<UserCourse>, (Status, ())> {
    let Authorization(user) = authorization;
    let Json(mut data) = data;
    data.user_id = user.id;
    Ok(UserCourseController::create_user_course(&context, data)
        .await
        .unwrap())
}

#[patch("/me/<id>", format = "json", data = "<data>")]
pub async fn update_user_course<'a>(
    context: Context<'a>,
    authorization: Authorization,
    id: &str,
    data: Json<UserCourse>,
) -> Result<Json<UserCourse>, (Status, ())> {
    let Authorization(user) = authorization;

    let filter = doc! {"_id": id,"user_id": user.id,};

    Ok(UserCourseController::update_user_course(
        &context,
        filter,
        Into::<UpdateModifications>::into(data.0),
    )
    .await
    .unwrap())
}
```