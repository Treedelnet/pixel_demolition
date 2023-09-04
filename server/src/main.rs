mod static_assets;
mod client;
mod engine;
mod game_matches; 
mod init_ws_handler;
mod etag;

use warp;
use warp::Filter;
use std::sync::Arc;

use static_assets::StaticAssets;
use game_matches::GameMatches;
use init_ws_handler::InitWSHandler;
use etag::Etag;


#[tokio::main]
async fn main() {
    let game_matches = GameMatches::new();
    let game_matches = Arc::new(game_matches);

    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || game_matches.clone()))
        .and_then(InitWSHandler::handle);

    let static_assets_route = warp::any()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::any().map(move || Etag::get()))
        .and_then(StaticAssets::handle);

    let routes = ws_route.or(static_assets_route);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

