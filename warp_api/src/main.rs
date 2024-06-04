use warp::Filter;
use serde::{Deserialize, Serialize};
use mongodb::{Client, options::ClientOptions, bson::{self, doc}, Collection};
use futures::stream::TryStreamExt;

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: Option<bson::oid::ObjectId>,
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() {
    // MongoDB Configurations
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("warp_api");
    let users_collection = db.collection::<User>("users");

    // Clone the collection handle to move into the closure
    let users_collection_filter = warp::any().map(move || users_collection.clone());

    // GET - Endpoint /hello
    let hello = warp::path("hello")
        .map(|| warp::reply::json(&"Hello, World!"));

    // POST - Endpoint /users
    let create_user = warp::path("users")
        .and(warp::post())
        .and(warp::body::json())
        .and(users_collection_filter.clone())
        .and_then(create_user_handler);

    // GET all Users -  Endpoint /users
    let get_users = warp::path("users")
        .and(warp::get())
        .and(users_collection_filter.clone())
        .and_then(get_users_handler);

    // Combina as rotas
    let routes = hello.or(create_user).or(get_users);

    // Start server
    warp::serve(routes)
        .run(([127, 0, 0, 1], 7777))
        .await;
}

async fn create_user_handler(new_user: User, users_collection: mongodb::Collection<User>) -> Result<impl warp::Reply, warp::Rejection> {
    let result = users_collection.insert_one(new_user, None).await;
    match result {
        Ok(inserted) => {
            let user_id = inserted.inserted_id.as_object_id().unwrap().to_hex();
            Ok(warp::reply::json(&doc! { "id": user_id }))
        }
        Err(_) => Err(warp::reject::custom(ServerError)),
    }
}

async fn get_users_handler(users_collection: mongodb::Collection<User>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut cursor = users_collection.find(None, None).await.unwrap();
    let mut users = Vec::new();
    while let Some(user) = cursor.try_next().await.unwrap() {
        users.push(user);
    }
    Ok(warp::reply::json(&users))
}

#[derive(Debug)]
struct ServerError;
impl warp::reject::Reject for ServerError {}
