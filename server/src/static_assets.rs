use std::sync::Arc;
use warp::{http::{HeaderMap, Response, HeaderValue}, path::FullPath};

pub struct StaticAssets {}

impl StaticAssets {
    pub async fn handle(path: FullPath, headers: HeaderMap<HeaderValue>, etag:Arc<String>)
        -> Result<impl warp::Reply, std::convert::Infallible> 
    {

        if headers.get("if-none-match") == Some(&HeaderValue::from_str(&etag).unwrap()) {
            pub const BLANK_BODY:&[u8] = b"";
            return Ok(Response::builder()
                .status(304)
                .header("ETag", etag.to_string())
                .body(BLANK_BODY)
                .unwrap());
        }

        const STATIC_FILES_DEF: [(&'static str, &'static str, &[u8]); 19] = [
            (
                "image/png",
                "background.png",
                include_bytes!("../static/background.png"),
            ),
            ("image/png", "map.png", include_bytes!("../static/map.png")),
            (
                "image/png",
                "bot_base.png",
                include_bytes!("../static/bot_base.png"),
            ),
            (
                "image/png",
                "dead_bot.png",
                include_bytes!("../static/dead_bot.png"),
            ),
            (
                "image/png",
                "minigun.png",
                include_bytes!("../static/minigun.png"),
            ),
            (
                "image/png",
                "glauncher.png",
                include_bytes!("../static/glauncher.png"),
            ),
            (
                "image/png",
                "grenade.png",
                include_bytes!("../static/grenade.png"),
            ),
            (
                "image/png",
                "grenade_icon.png",
                include_bytes!("../static/grenade_icon.png"),
            ),
            (
                "image/png",
                "bullet.png",
                include_bytes!("../static/bullet.png"),
            ),
            (
                "image/png",
                "bullet_icon.png",
                include_bytes!("../static/bullet_icon.png"),
            ),

            (
                "image/png",
                "reticle.png",
                include_bytes!("../static/reticle.png"),
            ),
            (
                "image/png",
                "health.png",
                include_bytes!("../static/health.png"),
            ),
            (
                "audio/mpeg",
                "minigun.mp3",
                include_bytes!("../static/minigun.mp3"),
            ),
            (
                "audio/mpeg",
                "glauncher.mp3",
                include_bytes!("../static/glauncher.mp3"),
            ),
            (
                "audio/mpeg",
                "bullet.mp3",
                include_bytes!("../static/bullet.mp3"),
            ),
            (
                "audio/mpeg",
                "grenade.mp3",
                include_bytes!("../static/grenade.mp3"),
            ),
            (
                "text/html",
                "",
                include_bytes!("../static/index.html"),
            ),
            (
                "text/javascript",
                "pixel_demolition_client.js",
                include_bytes!("../static/pixel_demolition_client.js"),
            ),
            (
                "application/wasm",
                "pixel_demolition_client_bg.wasm",
                include_bytes!("../static/pixel_demolition_client_bg.wasm"),
            ),
        ];

        let path = path.as_str();
        let path = &path[1..];

        for i in 0..STATIC_FILES_DEF.len() {
            if path == STATIC_FILES_DEF[i].1 {
                return Ok(Response::builder()
                    .status(200)
                    .header("Content-Type", STATIC_FILES_DEF[i].0)
                    .header("ETag", etag.to_string())
                    .body(STATIC_FILES_DEF[i].2)
                    .unwrap());
            }
        }
        let empty: &[u8] = b"";
        return Ok(Response::builder()
            .status(404)
            .header("Content-Type", "txt")
            .body(empty)
            .unwrap());
    }
}
