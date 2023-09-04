use crate::projectile::Projectile;

use std::str;

pub struct Proto {}

impl Proto {
    pub const TST_JOIN_EXISTING: u8 = 0x00;
    pub const TST_CREATE_NEW: u8 = 0x01;
    pub const TST_TOGGLE_READY: u8 = 0x02;
    pub const TST_NEW_POS: u8 = 0x03;
    pub const TST_NEW_ANGLE: u8 = 0x04;
    pub const TST_TAKE_WEAPON: u8 = 0x05;
    pub const TST_TRIGGER_PULLED: u8 = 0x06;
    pub const TST_TRIGGER_RELEASED: u8 = 0x07;

    // TCT (u8) + status (u8)
    pub const TCT_JOIN_EXISTING_RESULT: u8 = 0x80;
    pub const JOIN_EXISTING_RESULT_SUCCESS: u8 = 0x01;
    pub const JOIN_EXISTING_RESULT_BAD_CODE: u8 = 0x02;
    pub const JOIN_EXISTING_RESULT_SERVER_ERROR: u8 = 0x03;

    // TCT (u8) + status (u8)
    pub const TCT_CREATE_NEW_RESULT: u8 = 0x81;
    //pub const CREATE_NEW_RESULT_SUCCESS: u8 = 0x01;
    //pub const CREATE_NEW_RESULT_SERVER_ERROR: u8 = 0x03;
    pub const CREATE_NEW_RESULT_SUCCESS: u8 = 0xff;
    pub const CREATE_NEW_RESULT_SERVER_ERROR: u8 = 0x00;


    pub const TCT_PLAYER_LIST: u8 = 0x82;
    pub const TCT_TOGGLE_READY: u8 = 0x83;

    // TCT (u8) + state (u8) + player_index (u8)
    pub const TCT_START_GAME: u8 = 0x84;

    pub const TCT_NEW_POS: u8 = 0x85;
    pub const TCT_NEW_ANGLE: u8 = 0x86;
    pub const TCT_WEAPON_SPAWN: u8 = 0x87;
    pub const TCT_REMOVE_WEAPON: u8 = 0x88;
    pub const TCT_ASSIGN_WEAPON: u8 = 0x89;
    pub const TCT_NEW_PROJECTILE: u8 = 0x90;
    pub const TCT_DESTROY_PROJECTILE: u8 = 0x91;
    pub const TCT_PROJECTILE_EXPLOSION: u8 = 0x92;
    pub const TCT_UPDATE_HEALTH: u8 = 0x93;
    pub const TCT_REMOVE_AMMO: u8 = 0x94;
    pub const TCT_KILL_PLAYER: u8 = 0x95;
    pub const TCT_RESPAWN_PLAYER: u8 = 0x96;
    pub const TCT_GAME_OVER_STATS: u8 = 0x97;

    pub const SEPARATOR: u8 = 0x1E;

    pub const FALSE: u8 = 0x00;
    pub const TRUE: u8 = 0xff;

    pub fn get_type(message: &Vec<u8>) -> Result<u8, &'static str> {
        if message.len() > 0 {
            return Ok(message[0]);
        }

        return Err("Unable to get message type");
    }

    // +--------+
    // | Pasers |
    // +--------+

    #[cfg(not(target_family = "wasm"))]
    pub fn parse_tst_join_existing(
        message: &Vec<u8>,
        game_code_length: i32,
    ) -> Result<(String, String), &'static str> {
        let game_code_length = game_code_length as usize;

        if message.len() >= 1 + game_code_length + 1 {
            let code = str::from_utf8(&message[1..(game_code_length + 1)]);
            let name = str::from_utf8(&message[(game_code_length + 1)..]);

            if code.is_err() {
                return Err("Invalid code bytes");
            }

            if name.is_err() {
                return Err("Invalid name bytes");
            }

            return Ok((
                String::from(code.unwrap()),
                String::from(name.unwrap()),
            ));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_player_list(
        message: &Vec<u8>,
    ) -> Result<Vec<String>, &'static str> {
        if message.len() >= 2 {
            let mut player_names: Vec<String> = Vec::new();

            let mut name_buff: String = String::new();

            for i in 1..message.len() {
                if &message[i] == &Proto::SEPARATOR {
                    let new_player = String::from(name_buff.clone());
                    name_buff.clear();
                    player_names.push(new_player);

                } else {
                    let character = message[i].clone();
                    let character = &[character];
                    let character = str::from_utf8(character);
                    if character.is_err() {
                        return Err("Unable to parse name");
                    }
                    name_buff += character.unwrap();
                }
            }

            return Ok(player_names);
        }

        return Err("Message too short");
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn parse_tst_create_new(
        message: &Vec<u8>,
    ) -> Result<String, &'static str> {
        if message.len() > 1 {
            let name = str::from_utf8(&message[1..]);

            if name.is_err() {
                return Err("Invalid name bytes");
            }

            return Ok(String::from(name.unwrap()));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_join_existing_result(
        message: &Vec<u8>,
    ) -> Result<u8, &'static str> {
        if message.len() > 1 {
            return Ok(message[1]);
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_create_new_result(
        message: &Vec<u8>,
    ) -> Result<(u8, String), &'static str> {
        if message.len() > 2 {
            let status = message[1];
            let code = str::from_utf8(&message[2..]);

            if code.is_err() {
                return Err("Invalid code bytes");
            }

            return Ok((status, String::from(code.unwrap())))
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_toggle_ready(
        message: &Vec<u8>,
    ) -> Result<(usize, bool), &'static str> {
        if message.len() > 2 {
            let player_i = message[1] as usize;

            match message[2] {
                Proto::TRUE => {
                    return Ok((player_i, true));
                }
                Proto::FALSE => {
                    return Ok((player_i, false));
                }
                _ => {
                    return Err("Unknown ready state");
                }
            }
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_start_game(
        message: &Vec<u8>,
    ) -> Result<(usize, f32, f32), &'static str> {
        if message.len() >= 10 {
            let player_i = message[1] as usize;
            let x_bytes:[u8;4] = message[2..6].try_into().unwrap();
            let y_bytes:[u8;4] = message[6..10].try_into().unwrap();
            let x = f32::from_le_bytes(x_bytes);
            let y = f32::from_le_bytes(y_bytes);

            return Ok((player_i, x, y));
        }

        return Err("Message too short");
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn parse_tst_new_pos(
        message: &Vec<u8>,
    ) -> Result<(f32, f32), &'static str> {
        if message.len() >= 9 {
            let x_bytes:[u8;4] = message[1..5].try_into().unwrap();
            let y_bytes:[u8;4] = message[5..9].try_into().unwrap();

            let x = f32::from_le_bytes(x_bytes);
            let y = f32::from_le_bytes(y_bytes);

            return Ok((x, y));
        }

        return Err("Message too short");
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn parse_tst_new_angle (
        message: &Vec<u8>,
    ) -> Result<f32, &'static str> {
        if message.len() >= 5 {
            let angle_bytes:[u8;4] = message[1..5].try_into().unwrap();

            let angle = f32::from_le_bytes(angle_bytes);

            return Ok(angle);
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_new_pos(
        message: &Vec<u8>,
    ) -> Result<(usize, f32, f32), &'static str> {
        if message.len() >= 10 {
            let player_i = message[1] as usize;

            let x_bytes:[u8;4] = message[2..6].try_into().unwrap();
            let y_bytes:[u8;4] = message[6..10].try_into().unwrap();

            let x = f32::from_le_bytes(x_bytes);
            let y = f32::from_le_bytes(y_bytes);

            return Ok((player_i, x, y));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_new_angle(
        message: &Vec<u8>,
    ) -> Result<(usize, f32), &'static str> {
        if message.len() > 5 {
            let player_i = message[1] as usize;

            let angle_bytes:[u8;4] = message[2..6].try_into().unwrap();

            let angle = f32::from_le_bytes(angle_bytes);

            return Ok((player_i, angle));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_weapon_spawn(
        message: &Vec<u8>,
    ) -> Result<(usize, f32, f32), &'static str> {
        if message.len() >= 10 {
            let weapon_type = message[1] as usize;

            let x_bytes:[u8;4] = message[2..6].try_into().unwrap();
            let y_bytes:[u8;4] = message[6..10].try_into().unwrap();

            let x = f32::from_le_bytes(x_bytes);
            let y = f32::from_le_bytes(y_bytes);

            return Ok((weapon_type, x, y));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_remove_weapon(
        message: &Vec<u8>,
    ) -> Result<usize, &'static str> {
        if message.len() > 1 {
            let weapon_i = message[1] as usize;

            return Ok(weapon_i);
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_assign_weapon(
        message: &Vec<u8>,
    ) -> Result<(usize, usize), &'static str> {
        if message.len() > 2 {
            let player_i = message[1] as usize;
            let weapon_type = message[2] as usize;

            return Ok((player_i, weapon_type));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_new_projectile(
        message: &Vec<u8>,
    ) -> Result<(usize, f32, f32, f32, f32), &'static str> {
        if message.len() > 17 {
            let projectile_type = message[1] as usize;

            let x_bytes:[u8;4] = message[2..6].try_into().unwrap();
            let x = f32::from_le_bytes(x_bytes);
            let y_bytes:[u8;4] = message[6..10].try_into().unwrap();
            let y = f32::from_le_bytes(y_bytes);

            let vel_x_bytes:[u8;4] = message[10..14].try_into().unwrap();
            let vel_x = f32::from_le_bytes(vel_x_bytes);
            let vel_y_bytes:[u8;4] = message[14..18].try_into().unwrap();
            let vel_y = f32::from_le_bytes(vel_y_bytes);

            return Ok((projectile_type, x, y, vel_x, vel_y));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_projectile_explosion(
        message: &Vec<u8>,
    ) -> Result<Projectile, &'static str> {
        if message.len() >= 10 {
            let projectile_type = message[1] as usize;
            let x_bytes:[u8;4] = message[2..6].try_into().unwrap();
            let x = f32::from_le_bytes(x_bytes);
            let y_bytes:[u8;4] = message[6..10].try_into().unwrap();
            let y = f32::from_le_bytes(y_bytes);

            return Ok(Projectile {
                projectile_type: projectile_type,
                x,
                y,
                vel_x: 0.0,
                vel_y: 0.0,
                owner: 0,
            });
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_destroy_projectile(
        message: &Vec<u8>,
    ) -> Result<usize, &'static str> {
        if message.len() > 1 {
            let projectile_i:usize = message[1] as usize;
            return Ok(projectile_i);
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_update_health(
        message: &Vec<u8>,
    ) -> Result<f32, &'static str> {
        if message.len() >= 4 {
            let health_bytes:[u8;4] = message[1..5].try_into().unwrap();
            let health = f32::from_le_bytes(health_bytes);
            return Ok(health);
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_kill_player(
        message: &Vec<u8>,
    ) -> Result<usize, &'static str> {
        if message.len() > 1 {
            let player_i = message[1] as usize;
            return Ok(player_i);
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_respawn_player(
        message: &Vec<u8>,
    ) -> Result<(usize, f32, f32), &'static str> {
        if message.len() >= 10 {
            let player_i = message[1] as usize;
            let x_bytes:[u8;4] = message[2..6].try_into().unwrap();
            let y_bytes:[u8;4] = message[6..10].try_into().unwrap();
            let x = f32::from_le_bytes(x_bytes);
            let y = f32::from_le_bytes(y_bytes);

            return Ok((player_i, x, y));
        }

        return Err("Message too short");
    }

    #[cfg(target_family = "wasm")]
    pub fn parse_tct_game_over_stats(message: &Vec<u8>) -> Vec<(i32, i32)> {
        let mut kills_deaths: Vec<(i32, i32)> = Vec::new();

        for i in (1..(message.len()-1)).step_by(8) {
            let kills_bytes:[u8;4] = message[i..(i+4)].try_into().unwrap();
            let deaths_bytes:[u8;4] = message[(i+4)..(i+8)].try_into().unwrap();
            let kills = i32::from_le_bytes(kills_bytes);
            let deaths = i32::from_le_bytes(deaths_bytes);
            kills_deaths.push((kills, deaths));
        }

        return kills_deaths;
    }

    // +-------------+
    // | Serializers |
    // +-------------+

    #[cfg(target_family = "wasm")]
    pub fn tst_create_new(name: &String) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TST_CREATE_NEW);
        data.extend_from_slice(name.as_bytes());
        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_join_existing(code: &String, name: &String) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TST_JOIN_EXISTING);
        data.extend_from_slice(code.as_bytes());
        data.extend_from_slice(name.as_bytes());
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_join_existing_result(status: u8) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_JOIN_EXISTING_RESULT);
        data.push(status);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_create_new_result(status: u8, code: &String) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_CREATE_NEW_RESULT);
        data.push(status);
        data.extend_from_slice(code.as_bytes());
        return data;
    }


    #[cfg(not(target_family = "wasm"))]
    pub fn tct_player_list(names: &Vec<String>) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_PLAYER_LIST);

        for name in names {
            data.extend_from_slice(name.as_bytes());
            data.push(Self::SEPARATOR);
        }

        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_toggle_ready() -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TST_TOGGLE_READY);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_toggle_ready(player_i: usize, ready: bool) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_TOGGLE_READY);
        data.push(player_i as u8);

        match ready {
            true => data.push(Self::TRUE),
            false => data.push(Self::FALSE),
        }

        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_start_game(player_i: usize, player_x: f32, player_y: f32) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        data.push(Self::TCT_START_GAME);
        data.push(player_i as u8);
        data.extend_from_slice(&(player_x.to_le_bytes()));
        data.extend_from_slice(&(player_y.to_le_bytes()));

        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_new_pos(player_x: f32, player_y: f32) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();

        data.push(Proto::TST_NEW_POS);
        data.extend_from_slice(&(player_x.to_le_bytes()));
        data.extend_from_slice(&(player_y.to_le_bytes()));

        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_new_pos(player_i: usize, player_x: f32, player_y: f32) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();

        data.push(Proto::TCT_NEW_POS);
        data.push(player_i as u8);
        data.extend_from_slice(&(player_x.to_le_bytes()));
        data.extend_from_slice(&(player_y.to_le_bytes()));

        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_new_angle(player_i: usize, angle: f32) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();

        data.push(Proto::TCT_NEW_ANGLE);
        data.push(player_i as u8);
        data.extend_from_slice(&(angle.to_le_bytes()));

        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_new_angle(angle: f32) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();

        data.push(Proto::TST_NEW_ANGLE);
        data.extend_from_slice(&(angle.to_le_bytes()));

        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_take_weapon() -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        data.push(Proto::TST_TAKE_WEAPON);
        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_trigger_pulled() -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        data.push(Proto::TST_TRIGGER_PULLED);
        return data;
    }

    #[cfg(target_family = "wasm")]
    pub fn tst_trigger_released() -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        data.push(Proto::TST_TRIGGER_RELEASED);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_weapon_spawn(weapon_type: usize, x: f32, y: f32) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_WEAPON_SPAWN);
        data.push(weapon_type as u8);
        data.extend_from_slice(&x.to_le_bytes());
        data.extend_from_slice(&y.to_le_bytes());
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_remove_weapon(weapon_i: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_REMOVE_WEAPON);
        data.push(weapon_i as u8);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_assign_weapon(player_i: usize, weapon_type: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_ASSIGN_WEAPON);
        data.push(player_i as u8);
        data.push(weapon_type as u8);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_new_projectile(projectile: &Projectile) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_NEW_PROJECTILE);
        data.push(projectile.projectile_type as u8);
        data.extend_from_slice(&(projectile.x.to_le_bytes()));
        data.extend_from_slice(&(projectile.y.to_le_bytes()));
        data.extend_from_slice(&(projectile.vel_x.to_le_bytes()));
        data.extend_from_slice(&(projectile.vel_y.to_le_bytes()));
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_destroy_projectile(projectile_i: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_DESTROY_PROJECTILE);
        data.push(projectile_i as u8);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_projectile_explosion(projectile: &Projectile) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_PROJECTILE_EXPLOSION);
        data.push(projectile.projectile_type as u8);
        data.extend_from_slice(&(projectile.x.to_le_bytes()));
        data.extend_from_slice(&(projectile.y.to_le_bytes()));
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_update_health(health: f32) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_UPDATE_HEALTH);
        data.extend_from_slice(&(health.to_le_bytes()));
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_remove_ammo() -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_REMOVE_AMMO);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_kill_player(player_i: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_KILL_PLAYER);
        data.push(player_i as u8);
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_respawn_player(player_i: usize, x: f32, y: f32) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_RESPAWN_PLAYER);
        data.push(player_i as u8);
        data.extend_from_slice(&(x.to_le_bytes()));
        data.extend_from_slice(&(y.to_le_bytes()));
        return data;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn tct_game_over_stats(kills_deaths: &Vec<(i32, i32)>) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.push(Self::TCT_GAME_OVER_STATS);

        for i in 0..kills_deaths.len() {
            data.extend_from_slice(&(kills_deaths[i].0).to_le_bytes());
            data.extend_from_slice(&(kills_deaths[i].1).to_le_bytes());
        }

        return data;
    }
}
