use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use warp::ws::{Message, WebSocket, Ws};

use pixel_demolition_common::proto::Proto;

use crate::game_matches::game_match::GameMatch;
use crate::game_matches::GameMatches;
use crate::engine::Engine;

pub struct InitWSHandler {}

impl InitWSHandler {
    pub async fn handle(
        ws: Ws,
        game_matches: Arc<GameMatches>,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        println!("Upgrading websocket");
        Ok(ws.on_upgrade(move |socket| Self::handle_upgraded(socket, game_matches)))
    }

    pub async fn handle_upgraded(websocket: WebSocket, game_matches: Arc<GameMatches>) {
        let (mut websocket_send, mut websocket_recv) = websocket.split();

        // Loop until user joins an existing match or creates a new one
        while let Some(message) = websocket_recv.next().await {
            println!("Received message");
            if message.is_err() {
                eprintln!("{}", message.unwrap_err());
                continue;
            }

            let message = message.unwrap().into_bytes();

            let message_type = Proto::get_type(&message);

            if message_type.is_err() {
                continue;
            }

            match message_type.unwrap() {
                Proto::TST_JOIN_EXISTING => {
                    let join_message =
                        Proto::parse_tst_join_existing(&message, GameMatch::GAME_CODE_LEN);

                    if join_message.is_err() {
                        eprintln!("{}", join_message.unwrap_err());
                        continue;
                    }

                    let (code, name) = join_message.unwrap();

                    println!("Looking for match {}", code);

                    let game_match_i = game_matches.index_by_code(&code).await;

                    match game_match_i {
                        Ok(game_match_i) => {
                            println!("Successfully joined match {}", &code);
                            let join_result_message = Proto::tct_join_existing_result(
                                Proto::JOIN_EXISTING_RESULT_SUCCESS,
                            );
                            
                            let _ = websocket_send.send(Message::binary(join_result_message)).await;

                            game_matches.push_client(
                                game_match_i,
                                websocket_send,
                                websocket_recv,
                                name,
                            ).await;

                            return;
                        }
                        Err(err) => {
                            println!("Unable to find match {}", err);
                            let join_result_message = Proto::tct_join_existing_result(
                                Proto::JOIN_EXISTING_RESULT_BAD_CODE,
                            );
                            
                            let _ = websocket_send.send(Message::binary(join_result_message)).await;
                        }
                    }
                },
                Proto::TST_CREATE_NEW => {
                    println!("Received request to create new server");

                    let name = Proto::parse_tst_create_new(&message);

                    if name.is_err() {
                        eprintln!("{}", name.unwrap_err());
                        continue;
                    }

                    let name = name.unwrap();

                    let result = game_matches.activate().await;

                    match result {
                        Ok(game_match_i) => {
                            let code = &game_matches.c[game_match_i].read().await.code.clone();
                            let create_result_message = Proto::tct_create_new_result(
                                Proto::CREATE_NEW_RESULT_SUCCESS,
                                code,
                            );

                            println!("Created new server with code {}", code);
                            
                            let _ = websocket_send.send(Message::binary(create_result_message)).await;

                            println!("Sent message");

                            game_matches.push_client(
                                game_match_i,
                                websocket_send,
                                websocket_recv,
                                name,
                            ).await;

                            let game_matches_cloned = game_matches.clone();

                            tokio::spawn(async move {
                                println!("Retrieving match");
                                let game_match = &game_matches_cloned.c[game_match_i];
                                Engine::handle(game_match).await;
                            });
                            
                            println!("Spawned thread");

                            return;
                        },
                        Err(err) => {
                            println!("Unable to create server {}", err);
                            let create_result_message = Proto::tct_create_new_result(
                                Proto::CREATE_NEW_RESULT_SERVER_ERROR,
                                &String::new()
                            );
                            let _ = websocket_send.send(Message::binary(create_result_message)).await;
                        }
                    }
                },
                _ => (),
            }
        }
    }
}
