pub mod game_match;

use rand::{distributions::Alphanumeric, Rng};
use futures::stream::{SplitSink, SplitStream};
use tokio::sync::RwLock;
use warp::ws::{WebSocket, Message};
use tokio::time::{sleep, Duration};

use game_match::GameMatch;

pub struct GameMatches {
    pub c: Vec<RwLock<GameMatch>>,
}

impl GameMatches {
    pub const MAX_MATCHES: usize = 100;

    pub fn new() -> GameMatches {
        let mut c:Vec<RwLock<GameMatch>> = Vec::new();

        for _ in 0..Self::MAX_MATCHES {
            let new_game_match = GameMatch::new();
            let new_game_match = RwLock::new(new_game_match);

            c.push(new_game_match);
        }

        GameMatches {
            c
        }
    }

    pub async fn index_by_code(
        &self,
        code: &String, 
    ) -> Result<usize, &'static str> {
        for i in 0..Self::MAX_MATCHES {
            let game_match = self.c[i].read().await;
            if code == &game_match.code {
                return Ok(i as usize);
            }
        }
        return Err("No match");
    }

    pub async fn push_client(
        &self,
        game_match_i: usize,
        websocket_send: SplitSink<WebSocket, Message>,
        websocket_recv: SplitStream<WebSocket>,
        name: String,
    ) {
        let mut game_match = self.c[game_match_i].write().await;
        game_match.push_client(websocket_send, websocket_recv, name);
    }

    pub async fn activate(&self) -> Result<usize, &'static str> {
        // Find unused match in vector
        let mut reserved = Self::MAX_MATCHES + 1;

        for i in 0..Self::MAX_MATCHES {
            let mut game_match = self.c[i].write().await;

            if game_match.state == GameMatch::UNUSED {
                game_match.state = GameMatch::LOBBY;
                reserved = i;
                break;
            }
        }

        if reserved > Self::MAX_MATCHES {
            return Err("Match limit reached");
        }

        loop {
            let new_code: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(GameMatch::GAME_CODE_LEN as usize)
                .map(char::from)
                .collect();

            let new_code = new_code.to_uppercase();

            let mut existing_found = false;

            for i in 0..Self::MAX_MATCHES {
                if new_code == self.c[i].read().await.code {
                    existing_found = true;
                    break;
                }
            }
            if !existing_found {
                let mut game_match = self.c[reserved].write().await;
                game_match.code = new_code;

                return Ok(reserved);
            }

            sleep(Duration::from_millis(30)).await;
        }
    }

}
