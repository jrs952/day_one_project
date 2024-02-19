use warp::Filter;


#[tokio::main]
async fn main() {
    // Define a route
    let hello_route = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    // Start the Warp server
    warp::serve(hello_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
