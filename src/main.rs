use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use warp::{reply, Reply, Filter, reject, Rejection, http::StatusCode};
use std::fs;
use tokio::io::{AsyncWriteExt, AsyncReadExt};


#[derive(Serialize, Deserialize, Clone)]
struct Book {
    name: String,
    author: String,
    published: NaiveDate,
}

#[derive(Debug)]
struct InvalidParameter;

impl reject::Reject for InvalidParameter {}

// Custom rejection handler that maps rejections into responses.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if let Some(_e) = err.find::<InvalidParameter>() {
        Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(reply::with_status("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR))
    }
}

async fn save_book(book: &Book) -> Result<(), std::io::Error> {
    let file_name = format!("./books/{}.json", sanitize_filename(&book.name));    
    let data = serde_json::to_string(&book)?;    
    tokio::fs::create_dir_all("./books").await?; // Create directories recursively if they don't exist
    let mut file = tokio::fs::File::create(&file_name).await?;    
    file.write_all(data.as_bytes()).await?;
    Ok(())
}

async fn get_book(name: &str) -> Result<Book, std::io::Error> {
    let file_name = format!("./books/{}.json",sanitize_filename(name));
    let mut file = tokio::fs::File::open(file_name).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let book: Book = serde_json::from_str(&contents)?;
    Ok(book)
}

async fn delete_book(name: &str) -> Result<(), std::io::Error> {
    let file_name = format!("./books/{}.json",sanitize_filename(name));
    fs::remove_file(file_name)?;
    Ok(())
}

fn sanitize_filename(name: &str) -> String {    
    name.replace(" ", "_").replace("/", "_")
}



#[tokio::main]
async fn main() {
    let save = warp::post()
        .and(warp::path("save"))
        .and(warp::body::json())
        .and_then(|book: Book| async move {
            save_book(&book).await
                .map(|_| warp::reply::json(&"Book saved successfully"))
                .map_err(|_| warp::reject::custom(InvalidParameter))
        });

    let get = warp::get()
        .and(warp::path("get"))
        .and(warp::path::param())
        .and_then(|name: String| async move {
            get_book(&name).await
                .map(|book| warp::reply::json(&book))
                .map_err(|_| warp::reject::not_found())
        });

    let delete = warp::delete()
        .and(warp::path("delete"))
        .and(warp::path::param())
        .and_then(|name: String| async move {
            delete_book(&name).await
                .map(|_| warp::reply::with_status("Book deleted successfully", warp::http::StatusCode::OK))
                .map_err(|_| warp::reject::custom(InvalidParameter))
        });

    let routes = save.or(get).or(delete).recover(handle_rejection);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

