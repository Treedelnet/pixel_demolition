use tokio::sync::RwLock;

use futures::{FutureExt, SinkExt, StreamExt};
use image::{self, RgbaImage};
use rand::Rng;
use tokio::time::{self, Duration};
use warp::ws::Message;

use pixel_demolition_common::collision_system::CollisionSystem;
use pixel_demolition_common::player::Player;
use pixel_demolition_common::projectile::Projectile;
use pixel_demolition_common::proto::Proto;
use pixel_demolition_common::server_tick::ServerTick;
use pixel_demolition_common::vel_system::VelSystem;
use pixel_demolition_common::weapon::Weapon;

use crate::game_matches::game_match::GameMatch;

pub struct Engine {}

impl Engine {
    // Spawn new weapons every 30 seconds
    pub const WEAPONS_SPAWN_TICKS: i32 = 30 * (1000 as f32 / ServerTick::SERVER_TICK as f32) as i32;
    pub const SPAWN_WEAPONS_COUNT: usize = 10;
    pub const PICKUP_RANGE: f32 = 20.0;
    pub const KILLS_TO_WIN_GAME: i32 = 5;

    const CLEAR_PIXEL: image::Rgba<u8> = image::Rgba::<u8>([0, 0, 0, 0]);

    // Run collision checks for projectiles 10 times per tick
    const PROJECTILE_INTERP_RATE: i32 = 10;

    // Timeout after 30 minutes
    const TIMEOUT: i32 = 30*60*(1000/ServerTick::SERVER_TICK);

    pub async fn handle(game_match: &RwLock<GameMatch>) {
        let result = Self::lobby(game_match).await;

        if let Err(error) = result {
            println!("Releasing lobby early: {}", error);
            Self::release_match(game_match).await;
            return;
        }

        let mut players = result.unwrap();

        let result = Self::main(game_match, &mut players).await;

        if let Err(error) = result {
            println!("Releasing match early: {}", error);
            Self::release_match(game_match).await;
            return;
        }

        Self::game_over(game_match, &mut players).await;
        Self::release_match(game_match).await;
        println!("Released game match");
    }

    pub async fn lobby(game_match: &RwLock<GameMatch>) -> Result<Vec<Player>, &'static str> {
        let mut client_len = 0;

        let mut players: Vec<Player> = Vec::new();

        println!("In lobby");

        let mut interval = time::interval(Duration::from_millis(ServerTick::SERVER_TICK as u64));

        let mut ticks_alive = 0;

        loop {
            interval.tick().await;
            {
                let mut game_match = game_match.write().await;

                if game_match.clients.len() < 1 {
                    continue;
                }

                // Broadcast new players
                if game_match.clients.len() > client_len {
                    for client_i in client_len..game_match.clients.len() {
                        let new_player = Player::new(game_match.clients[client_i].name.clone());
                        players.push(new_player);
                    }

                    client_len = game_match.clients.len();

                    let mut names: Vec<String> = Vec::new();

                    for client in &game_match.clients {
                        names.push(client.name.clone());
                    }

                    let player_list_message = Proto::tct_player_list(&names);

                    for client in &mut game_match.clients {
                        let websocket_send = &mut client.websocket_send;
                        let _ = websocket_send
                            .send(Message::binary(player_list_message.clone()))
                            .await;
                    }
                }

                // Handle toggle messages
                for client_i in 0..game_match.clients.len() {
                    let client = &mut game_match.clients[client_i];
                    let player = &mut players[client_i];

                    let message = client.websocket_recv.next().now_or_never();

                    // Continue now_or_never() returned None
                    if message.is_none() {
                        continue;
                    }

                    let message = message.unwrap();

                    // Continue if there was no valid websocket message 
                    if message.is_none() {
                        continue;
                    }

                    let message = message.unwrap();

                    // Continue if there was a websocket error
                    if message.is_err() {
                        continue;
                    }

                    let message = message.unwrap().into_bytes();

                    let message_type = Proto::get_type(&message);

                    if message_type.is_err() {
                        continue;
                    }

                    if message_type.unwrap() == Proto::TST_TOGGLE_READY {
                        player.ready = !player.ready;
                        let toggle_message = Proto::tct_toggle_ready(client_i, player.ready);
                        for client in &mut game_match.clients {
                            let websocket_send = &mut client.websocket_send;
                            let _ = websocket_send
                                .send(Message::binary(toggle_message.clone()))
                                .await;
                        }
                    }
                }

                // Check to see if all players are ready
                let mut ready = true;
                for player in &players {
                    if !player.ready {
                        ready = false;
                    }
                }

                if ready {
                    return Ok(players);
                }

            }

            if ticks_alive > Self::TIMEOUT {
                return Err("Lobby timeout");
            }
            ticks_alive += 1;

        }
    }

    pub async fn main(game_match: &RwLock<GameMatch>, players: &mut Vec<Player>)
        -> Result<(), &'static str>
    {
        // Once the match starts we can permanently lock it since clients won't be added and
        // InitWSHandler can still read() it for checking the status
        let game_match = &mut game_match.write().await;
        game_match.state = GameMatch::GAME;

        // Create a fresh copy of the map
        let map_bytes = include_bytes!("../static/map.png");
        let dynamic_map_image = image::load_from_memory(map_bytes);
        let mut map = dynamic_map_image.unwrap().into_rgba8();

        let mut projectiles: Vec<Projectile> = Vec::new();
        let mut ground_weapons: Vec<Weapon> = Vec::new();
        let mut ticks_since_weapon_spawn: i32 = -1;

        let mut messages: Vec<Vec<u8>> = Vec::new();

        for client_i in 0..game_match.clients.len() {
            let (x, y) = Self::get_rand_pos(&map);
            players[client_i].x = x;
            players[client_i].y = y;

            let start_message = Proto::tct_start_game(client_i, x, y);

            let client = &mut game_match.clients[client_i];
            let websocket_send = &mut client.websocket_send;

            let _ = websocket_send
                .send(Message::binary(start_message.clone()))
                .await;
        }

        let mut ticks_alive = 0;

        let mut interval = time::interval(Duration::from_millis(ServerTick::SERVER_TICK as u64));

        'game_loop: loop {
            interval.tick().await;

            for player_i in 0..players.len() {
                messages.clear();

                if !game_match.clients[player_i].connected {
                    continue;
                }

                loop {
                    let message = game_match.clients[player_i]
                        .websocket_recv
                        .next()
                        .now_or_never();

                    // Break if no messages in the queue
                    if message.is_none() {
                        break;
                    }

                    let message = message.unwrap();

                    // Break if the socket closed
                    if message.is_none() {
                        game_match.clients[player_i].connected = false;
                        break;
                    }

                    let message = message.unwrap();

                    // Continue if there was a websocket error
                    if message.is_err() {
                        println!("{}", message.unwrap_err());
                        continue;
                    }

                    let message = message.unwrap().into_bytes();

                    messages.push(message);
                }

                let mut set_angle = false;
                let mut set_pos = false;

                // Iterate over the newest messages first
                for message in (&messages).into_iter().rev() {
                    let message_type = Proto::get_type(&message);

                    if message_type.is_err() {
                        continue;
                    }

                    match message_type.unwrap() {
                        Proto::TST_NEW_POS => {
                            // If the player position has already been updated with a newer message,
                            // ignore this one
                            if set_pos {
                                continue;
                            }

                            Self::handle_pos_update(&message, game_match, players, player_i, &map)
                                .await;

                            set_pos = true;
                        }

                        Proto::TST_NEW_ANGLE => {
                            // If the player angle has already been updated with a newer message,
                            // ignore this one
                            if set_angle {
                                continue;
                            }

                            Self::handle_angle_update(&message, game_match, players, player_i)
                                .await;

                            set_angle = true;
                        }
                        Proto::TST_TAKE_WEAPON => {
                            let player = &mut players[player_i];
                            Self::handle_take_weapon(
                                game_match,
                                player,
                                player_i,
                                &mut ground_weapons,
                            )
                            .await;
                        }
                        Proto::TST_TRIGGER_PULLED => {
                            players[player_i].trigger_pulled = true;
                        }
                        Proto::TST_TRIGGER_RELEASED => {
                            players[player_i].trigger_pulled = false;
                        }
                        _ => (),
                    }
                }
            }

            Self::handle_weapons(game_match, players, &mut projectiles).await;
            Self::handle_projectiles(game_match, players, &mut projectiles, &mut map).await;

            Self::handle_player_respawns(game_match, players, &mut map).await;

            if ticks_since_weapon_spawn < 0 || ticks_since_weapon_spawn >= Self::WEAPONS_SPAWN_TICKS
            {
                Self::handle_weapon_spawns(game_match, &mut ground_weapons, &map).await;
                ticks_since_weapon_spawn = 0;
            } else {
                ticks_since_weapon_spawn += 1;
            }

            for player_i in 0..players.len() {
                if players[player_i].kills >= Self::KILLS_TO_WIN_GAME {
                    break 'game_loop;
                }
            }

            let mut any_connected = false;
            for client in &game_match.clients {
                if client.connected {
                    any_connected = true;
                    break;
                }
            }

            if !any_connected {
                return Err("All clients disconnected");
            }

            if ticks_alive > Self::TIMEOUT {
                return Err("Game timeout");

            } 
            ticks_alive += 1;
        }

        return Ok(());
    }

    pub async fn game_over(game_match: &RwLock<GameMatch>, players: &mut Vec<Player>) {
        let game_match = &mut game_match.write().await;

        let mut kills_deaths: Vec<(i32, i32)> = Vec::new();

        for player in players {
            kills_deaths.push((player.kills, player.deaths));
        }

        let game_over_stats_message = Proto::tct_game_over_stats(&kills_deaths);

        for client in &mut game_match.clients {
            let websocket_send = &mut client.websocket_send;
            let _ = websocket_send
                .send(Message::binary(game_over_stats_message.clone()))
                .await;
        }
    }

    pub async fn release_match(game_match: &RwLock<GameMatch>) {
        let game_match = &mut game_match.write().await;

        for client in &mut game_match.clients {
            let websocket_send = &mut client.websocket_send;
            let _ = websocket_send.close().await;
        }

        game_match.clients.clear();

        game_match.state = GameMatch::UNUSED;
    }

    pub async fn handle_pos_update(
        message: &Vec<u8>,
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        player_i: usize,
        map: &RgbaImage,
    ) {
        let message = Proto::parse_tst_new_pos(&message);

        if message.is_err() {
            println!("{}", message.unwrap_err());
            return;
        }

        let (x, y) = message.unwrap();

        players[player_i].x = x;
        players[player_i].y = y;

        let player_pos_message = Proto::tct_new_pos(player_i, x, y);

        for client_i in 0..game_match.clients.len() {
            // Don't send the player's position back to them
            if client_i == player_i {
                continue;
            }

            let websocket_send = &mut game_match.clients[client_i].websocket_send;

            let _ = websocket_send
                .send(Message::binary(player_pos_message.clone()))
                .await;
        }

        if CollisionSystem::player_oob(&players[player_i], map) {
            Self::handle_player_death(game_match, players, player_i, None).await;
        }
    }

    pub async fn handle_angle_update(
        message: &Vec<u8>,
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        player_i: usize,
    ) {
        let angle = Proto::parse_tst_new_angle(&message);

        if angle.is_err() {
            println!("{}", angle.unwrap_err());
            return;
        }

        let angle = angle.unwrap();

        players[player_i].angle = angle;

        let player_angle_message = Proto::tct_new_angle(player_i, angle);

        for client_i in 0..game_match.clients.len() {
            // Don't send the player's angle back to them
            if client_i == player_i {
                continue;
            }

            let websocket_send = &mut game_match.clients[client_i].websocket_send;

            let _ = websocket_send
                .send(Message::binary(player_angle_message.clone()))
                .await;
        }
    }

    pub async fn handle_take_weapon(
        game_match: &mut GameMatch,
        player: &mut Player,
        player_i: usize,
        ground_weapons: &mut Vec<Weapon>,
    ) {
        for weapon_i in 0..ground_weapons.len() {
            if (player.x - ground_weapons[weapon_i].x).abs() < Self::PICKUP_RANGE
                && (player.y - ground_weapons[weapon_i].y).abs() < Self::PICKUP_RANGE
            {
                let weapon_type = ground_weapons[weapon_i].weapon_type;

                ground_weapons.remove(weapon_i as usize);

                let remove_weapon_message = Proto::tct_remove_weapon(weapon_i);
                let assign_weapon_message = Proto::tct_assign_weapon(player_i, weapon_type);

                player.assign_weapon(weapon_type);

                for client in &mut game_match.clients {
                    let websocket_send = &mut client.websocket_send;
                    let _ = websocket_send
                        .send(Message::binary(remove_weapon_message.clone()))
                        .await;

                    let _ = websocket_send
                        .send(Message::binary(assign_weapon_message.clone()))
                        .await;
                }

                break;
            }
        }
    }

    pub async fn handle_weapons(
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        projectiles: &mut Vec<Projectile>,
    ) {
        for player_i in 0..players.len() {
            let player = &mut players[player_i];

            if player.weapon_type.is_none() {
                continue;
            }

            let weapon_type = player.weapon_type.unwrap();

            let ticks_per_fire = Weapon::WEAPON_TYPES[weapon_type].ticks_per_fire;

            if player.ticks_since_last_fire > -1 && player.ticks_since_last_fire <= ticks_per_fire {
                player.ticks_since_last_fire += 1;
                continue;
            }

            if player.trigger_pulled == false {
                continue;
            }

            if player.ammo <= 0 {
                continue;
            }

            let projectile_type = Weapon::WEAPON_TYPES[weapon_type].projectile_type;

            let init_vel = Projectile::PROJECTILE_TYPES[projectile_type].init_vel;

            let vel_x = -player.angle.cos() * init_vel;
            let vel_y = -player.angle.sin() * init_vel;

            let offset_x = -player.angle.cos() * (Weapon::WEAPON_WIDTH) as f32;
            let offset_y = -player.angle.sin() * (Weapon::WEAPON_WIDTH) as f32;

            let init_x = player.x + offset_x;
            let init_y = player.y + offset_y;

            let new_projectile = Projectile {
                projectile_type,
                x: init_x as f32,
                y: init_y as f32,
                vel_x,
                vel_y,
                owner: player_i,
            };

            let projectile_message = Proto::tct_new_projectile(&new_projectile);

            projectiles.push(new_projectile);

            for client_i in 0..game_match.clients.len() {
                let client = &mut game_match.clients[client_i];
                let websocket_send = &mut client.websocket_send;
                let _ = websocket_send
                    .send(Message::binary(projectile_message.clone()))
                    .await;

                if client_i == player_i {
                    let remove_ammo_message = Proto::tct_remove_ammo();

                    let _ = websocket_send
                        .send(Message::binary(remove_ammo_message.clone()))
                        .await;
                }
            }

            player.ticks_since_last_fire = 0;

            player.ammo -= 1;
        }
    }

    pub async fn handle_projectiles(
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        projectiles: &mut Vec<Projectile>,
        map: &mut RgbaImage,
    ) {
        // Track destroyed projectiles as we iterate
        let mut destroyed_projectiles: usize = 0;

        'per_projectile: for projectile_i in 0..projectiles.len() {
            // We will be popping projectiles off the Vec as we go so adjust based on that
            let projectile_i = projectile_i - destroyed_projectiles;

            if projectile_i >= projectiles.len() {
                break;
            }

            // Too much time passes during each tick to do good collisions, so interpolate
            for _ in 0..(Self::PROJECTILE_INTERP_RATE as usize) {
                VelSystem::update_projectile(
                    &mut projectiles[projectile_i],
                    (ServerTick::SERVER_TICK / Self::PROJECTILE_INTERP_RATE) as f32,
                );

                // If the projectile fell off the screen destroy it without an explosion
                if CollisionSystem::projectile_oob(&projectiles[projectile_i], map) {
                    let destroy_projectile_message = Proto::tct_destroy_projectile(projectile_i);

                    for client in &mut game_match.clients {
                        let websocket_send = &mut client.websocket_send;
                        let _ = websocket_send
                            .send(Message::binary(destroy_projectile_message.clone()))
                            .await;
                    }

                    println!("Projectile out of bounds");

                    projectiles.remove(projectile_i);
                    destroyed_projectiles += 1;

                    continue 'per_projectile;
                }

                if CollisionSystem::projectile_collide_map(&projectiles[projectile_i], map) {
                    Self::handle_projectile_damage(
                        game_match,
                        players,
                        &mut projectiles[projectile_i],
                        projectile_i,
                        map,
                    )
                    .await;

                    projectiles.remove(projectile_i);
                    destroyed_projectiles += 1;
                    continue 'per_projectile;
                }

                for player_i in 0..players.len() {
                    if CollisionSystem::point_collide_player(
                        projectiles[projectile_i].x,
                        projectiles[projectile_i].y,
                        &players[player_i],
                    ) {
                        Self::handle_projectile_damage(
                            game_match,
                            players,
                            &mut projectiles[projectile_i],
                            projectile_i,
                            map,
                        )
                        .await;

                        projectiles.remove(projectile_i);
                        destroyed_projectiles += 1;
                        continue 'per_projectile;
                    }
                }
            }
        }
    }

    pub async fn handle_projectile_damage(
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        projectile: &mut Projectile,
        projectile_i: usize,
        map: &mut RgbaImage,
    ) {
        let destroy_pixels = projectile.draw_explosion();

        // Keep track of which players were damaged so we can tell them their new health
        let mut players_health_affected: Vec<usize> = Vec::new();

        for destroy_pixel in destroy_pixels {
            let (destroy_pixel_x, destroy_pixel_y) = destroy_pixel;

            for player_i in 0..players.len() {
                if CollisionSystem::point_collide_player(
                    destroy_pixel_x as f32,
                    destroy_pixel_y as f32,
                    &players[player_i],
                ) {
                    if !players[player_i].alive {
                        continue;
                    }

                    let projectile_type = projectile.projectile_type;
                    let projectile_damage = Projectile::PROJECTILE_TYPES[projectile_type].damage;

                    players[player_i].health -= projectile_damage;

                    if !players_health_affected.contains(&player_i) {
                        players_health_affected.push(player_i);
                    }

                    if players[player_i].health < 0.0 {
                        Self::handle_player_death(
                            game_match,
                            players,
                            player_i,
                            Some(projectile.owner),
                        )
                        .await;
                        continue;
                    }
                }
            }

            if map.get_pixel(destroy_pixel_x as u32, destroy_pixel_y as u32)[3] > 0 {
                if destroy_pixel_x < 0 || destroy_pixel_x >= map.width() as i32 {
                    continue;
                } else if destroy_pixel_y < 0 || destroy_pixel_y >= map.height() as i32 {
                    continue;
                }

                map.put_pixel(
                    destroy_pixel_x as u32,
                    destroy_pixel_y as u32,
                    Self::CLEAR_PIXEL.clone(),
                );
            }
        }

        let projectile_explosion_message = Proto::tct_projectile_explosion(&projectile);
        let destroy_projectile_message = Proto::tct_destroy_projectile(projectile_i);

        for client_i in 0..game_match.clients.len() {
            let websocket_send = &mut game_match.clients[client_i].websocket_send;
            let _ = websocket_send
                .send(Message::binary(projectile_explosion_message.clone()))
                .await;
            let _ = websocket_send
                .send(Message::binary(destroy_projectile_message.clone()))
                .await;

            if players_health_affected.contains(&client_i) {
                let update_health_message = Proto::tct_update_health(players[client_i].health);

                let _ = websocket_send
                    .send(Message::binary(update_health_message.clone()))
                    .await;
            }
        }
    }

    pub async fn handle_player_death(
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        killed_player_i: usize,
        projectile_owner: Option<usize>,
    ) {
        println!("Killing player {}", killed_player_i);

        if projectile_owner.is_some() && projectile_owner.unwrap() != killed_player_i {
            players[projectile_owner.unwrap()].kills += 1;
        }

        players[killed_player_i].kill();

        let player_kill_message = Proto::tct_kill_player(killed_player_i);

        for client in &mut game_match.clients {
            let websocket_send = &mut client.websocket_send;
            let _ = websocket_send
                .send(Message::binary(player_kill_message.clone()))
                .await;
        }
    }

    pub async fn handle_player_respawns(
        game_match: &mut GameMatch,
        players: &mut Vec<Player>,
        map: &mut RgbaImage,
    ) {
        for player_i in 0..players.len() {
            let player = &mut players[player_i];

            if player.alive {
                continue;
            }

            player.time_to_respawn -= ServerTick::SERVER_TICK;

            if player.time_to_respawn > 0 {
                continue;
            }

            let (x, y) = Self::get_rand_pos(&map);
            player.respawn(x, y);

            let respawn_player_message = Proto::tct_respawn_player(player_i, x, y);

            for client in &mut game_match.clients {
                let websocket_send = &mut client.websocket_send;
                let _ = websocket_send
                    .send(Message::binary(respawn_player_message.clone()))
                    .await;
            }
        }
    }

    pub async fn handle_weapon_spawns(
        game_match: &mut GameMatch,
        ground_weapons: &mut Vec<Weapon>,
        map: &RgbaImage,
    ) {
        for _ in 0..Self::SPAWN_WEAPONS_COUNT {
            if ground_weapons.len() > 255 {
                return;
            }

            let (x, y) = Self::get_rand_pos(&map);
            let weapon_type = rand::thread_rng().gen_range(0..Weapon::WEAPON_TYPES.len());

            let weapon_type = weapon_type as usize;

            let new_weapon = Weapon::new(weapon_type, x, y);

            ground_weapons.push(new_weapon);

            let weapon_spawn_message = Proto::tct_weapon_spawn(weapon_type, x, y);

            for client in &mut game_match.clients {
                let websocket_send = &mut client.websocket_send;
                let _ = websocket_send
                    .send(Message::binary(weapon_spawn_message.clone()))
                    .await;
            }
        }
    }

    fn get_rand_pos(map: &RgbaImage) -> (f32, f32) {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(0..map.width() as usize);
            let y = rng.gen_range(0..map.height() as usize);
            let x = x as u32;
            let mut y = y as u32;
            while y < map.height() {
                // Find the ground
                if map.get_pixel(x, y)[3] > 0 {
                    // Move up to find the space right above the ground
                    while map.get_pixel(x, y)[3] > 0 {
                        y -= 1;
                    }
                    // Move it up to half the character's height
                    y -= Player::PLAYER_HEIGHT / 2;
                    return (x as f32, y as f32);
                }
                y += 1;
            }
            // If a location wasn't found that means the coordinates didn't have any ground below
            // them, so start over with new coordinates
        }
    }
}
