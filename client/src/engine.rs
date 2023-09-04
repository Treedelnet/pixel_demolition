use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;
use js_sys::Date;
use std::str;

use pixel_demolition_common::player::Player;
use pixel_demolition_common::projectile::Projectile;
use pixel_demolition_common::weapon::Weapon;
use pixel_demolition_common::proto::Proto;
use pixel_demolition_common::vel_system::VelSystem;
use pixel_demolition_common::collision_system::CollisionSystem;

use crate::input::Input;
use crate::graphics::Graphics;
use crate::flash::Flash;
use crate::connection::Connection;
use crate::states::States;
use crate::selected::Selected;
use crate::audio::{Audio, sounds::Sounds};
use crate::interp_system::InterpSystem;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Engine {
    window: web_sys::Window,
    graphics: Graphics,
    input: Input,
    connection: Option<Connection>,
    audio: Option<Audio>,
    state:States,
    state_changed:bool,
    players:Vec<Player>,
    ground_weapons: Vec<Weapon>,
    projectiles: Vec<Projectile>,
    player_i: Option<usize>,
    selected: Selected,
    last_time: f64,
    name: String,
    code: String,
    jump_debounce: bool,
    next_angle_update: f32,
    // x, y, radius, frames left to live
    flashes: Vec<Flash>,
}

// Send new angles 5/sec
const ANGLE_UPDATE_PERIOD: f32 = 200.0;

#[wasm_bindgen]
impl Engine {
    pub fn new() -> Engine {
        let window = web_sys::window().unwrap();

        console_error_panic_hook::set_once();

        let last_time = Date::now();
        
        let graphics = Graphics::new(&window);
        let input = Input::new();

        return Engine {
            window,
            graphics,
            input,
            connection: None,
            audio: None,
            players: Vec::new(),
            ground_weapons: Vec::new(),
            projectiles: Vec::new(),
            player_i: None,
            state: States::Unmatched,
            state_changed: true,
            selected: Selected::Name,
            last_time,
            name: String::new(),
            code: String::new(),
            jump_debounce: false,
            next_angle_update: 0.0,
            flashes: Vec::new(),
        };
    }

    pub async fn run(&mut self) {
        let current_time = Date::now();
        let time_elapsed = current_time - self.last_time;
        let time_elapsed = time_elapsed as f32;
        self.last_time = current_time;

        match self.state {
            States::Unmatched => {
                self.unmatched().await;
            }
            States::Lobby => {
                self.lobby();
            },
            States::Game => {
                self.game(time_elapsed).await;
            },
            States::GameOver => {
                self.game_over();
            }
        }
    }

    pub async fn unmatched(&mut self) {
        if self.state_changed {
            self.graphics.update_canvas(States::Unmatched);
            self.graphics.first_render_unmatched();
            self.state_changed=false;
        }

        let (_, mouse_coord_y) = self.input.mouse_coordinates();
        let mouse_clicked = self.input.mouse_clicked();

        if self.connection.is_some() {
            while let Some(message) = self.connection.as_mut().unwrap().next_message() {
                let message_type = Proto::get_type(&message);

                if message_type.is_err() {
                    continue;
                }

                match message_type.unwrap() {
                    Proto::TCT_JOIN_EXISTING_RESULT => {
                        let join_existing_result = Proto::parse_tct_join_existing_result(&message);

                        if join_existing_result.is_err() {
                            continue;
                        }

                        match join_existing_result.unwrap() {
                            Proto::JOIN_EXISTING_RESULT_SUCCESS => {
                                self.state_changed = true;
                                self.state = States::Lobby;
                                break;
                            },
                            Proto::JOIN_EXISTING_RESULT_BAD_CODE => {
                                self.graphics.update_render_unmatched_name_code(
                                    &self.name, &String::from("Invalid code")
                                );
                                self.connection.as_mut().unwrap().disconnect();
                                self.connection = None;
                            },
                            Proto::JOIN_EXISTING_RESULT_SERVER_ERROR | _ => {
                               self.graphics.update_render_unmatched_name_code(
                                    &self.name, &String::from("Server error")
                                );
                                self.connection.as_mut().unwrap().disconnect();
                                self.connection = None;
                            }
                        }
                    },
                    Proto::TCT_CREATE_NEW_RESULT => {
                        let create_new_result = Proto::parse_tct_create_new_result(&message);

                        if create_new_result.is_err() {
                            continue;
                        }

                        let (status, code) = create_new_result.unwrap();

                        match status {
                            Proto::CREATE_NEW_RESULT_SUCCESS => {
                                self.code = code;
                                self.state_changed = true;
                                self.state = States::Lobby;
                                break;
                            }
                            Proto::CREATE_NEW_RESULT_SERVER_ERROR | _ => {
                               self.graphics.update_render_unmatched_name_code(
                                    &self.name, &String::from("Invalid code")
                                );
                                self.connection.as_mut().unwrap().disconnect();
                                self.connection = None;
                            }
                        }
                    }
                    _ => ()
                }
            }
        }

        match mouse_coord_y {
            y if y > self.graphics.height_divided*2 && y < self.graphics.height_divided*3 => {
                self.selected = Selected::Name;
            },
            y if y > self.graphics.height_divided*5 && y < self.graphics.height_divided*6 => {
                self.selected = Selected::Code;
            },
            y if y > self.graphics.height_divided*6 && y < self.graphics.height_divided*7 => {
                if mouse_clicked {
                    let join_existing_message = Proto::tst_join_existing(&self.code, &self.name);
                    self.connection = Some(Connection::new(&self.window).await);
                    self.connection.as_mut().unwrap().send(join_existing_message);
                }
            },
            y if y > self.graphics.height_divided*8 && y < self.graphics.height_divided*9 => {
                if mouse_clicked {
                    let create_new_message = Proto::tst_create_new(&self.name);
                    self.connection = Some(Connection::new(&self.window).await);
                    self.connection.as_mut().unwrap().send(create_new_message);
                }
            },
            y if y > self.graphics.height_divided*10 && y < self.graphics.height_divided*11 => {
                if mouse_clicked && self.audio.is_none() {
                    self.audio = Some(Audio::new(&self.window).await);
                    self.graphics.update_render_unmatched_audio_enabled();
                }
            }
            _ => ()
        }

            
        for key in self.input.get_typed_keys() {
            let selected_data = match self.selected {
                Selected::Name => &mut self.name,
                Selected::Code => &mut self.code,
            };

            match key {
                // Alphanumeric characters 
                0x41..=0x90 | 0x30..=0x39 => {
                    selected_data.push_str(str::from_utf8(&[key]).unwrap());
                },
                0x08 => {
                    selected_data.pop();
                }
                _ => ()
            }

            self.graphics.update_render_unmatched_name_code(&self.name, &self.code);
        }
    }

    pub fn lobby(&mut self) {
        let (_, mouse_coord_y) = self.input.mouse_coordinates();
        let mouse_clicked = self.input.mouse_clicked();

        if self.state_changed {
            self.graphics.render_lobby(&self.code, &self.players);
            self.state_changed=false;
        }

        if mouse_coord_y > self.graphics.height_divided*8
            && mouse_coord_y < self.graphics.height_divided*9
            && mouse_clicked 
        {
            let toggle_ready_message = Proto::tst_toggle_ready();
            self.connection.as_mut().unwrap().send(toggle_ready_message);
        }

        while let Some(message) = self.connection.as_mut().unwrap().next_message() {
            let message_type = Proto::get_type(&message);

            if message_type.is_err() {
                continue;
            }

            match message_type.unwrap() {
                Proto::TCT_PLAYER_LIST => {
                    self.players.clear();

                    let player_names = Proto::parse_tct_player_list(&message);

                    if player_names.is_err() {
                        continue;
                    }

                    let player_names = player_names.unwrap();

                    self.players.clear();

                    for player_name in player_names {
                        self.players.push(Player::new(player_name));
                    }


                    self.graphics.render_lobby(&self.code, &self.players);
                },
                Proto::TCT_TOGGLE_READY => {
                    let toggle_message = Proto::parse_tct_toggle_ready(&message);

                    if toggle_message.is_err() {
                        continue;
                    }

                    let (player_i, ready) = toggle_message.unwrap();

                    self.players[player_i].ready = ready;

                    self.graphics.render_lobby(&self.code, &self.players);
                },
                Proto::TCT_START_GAME => {
                    let (player_i, x, y) = Proto::parse_tct_start_game(&message).unwrap();
                    self.player_i = Some(player_i);
                    self.players[player_i].x = x;
                    self.players[player_i].y = y;

                    self.state = States::Game;
                    self.state_changed = true;
                    break;
                }
                _ => ()
            }
        }
    }

    pub async fn game(&mut self, time_elapsed: f32) {
        if self.state_changed {
            self.graphics.update_canvas(States::Game);
            self.state_changed=false;
        }

        let (mouse_coord_x, mouse_coord_y) = self.input.mouse_coordinates();

        'game_loop: while let Some(message) = self.connection.as_mut().unwrap().next_message() {
            let message_type = Proto::get_type(&message);

            if message_type.is_err() {
                continue;
            }

            match message_type.unwrap() {
                Proto::TCT_NEW_POS => {
                    let result = Proto::parse_tct_new_pos(&message);

                    if result.is_err() {
                        log(result.unwrap_err());
                        continue;
                    }

                    let (player_i, x, y) = result.unwrap();

                    self.players[player_i].x_last = self.players[player_i].x;
                    self.players[player_i].y_last = self.players[player_i].y;

                    self.players[player_i].x_new = x;
                    self.players[player_i].y_new = y;
                },
                Proto::TCT_NEW_ANGLE => {
                    let result = Proto::parse_tct_new_angle(&message);

                    if result.is_err() {
                        continue;
                    }

                    let (player_i, angle) = result.unwrap();

                    self.players[player_i].angle = angle;
                },
                Proto::TCT_WEAPON_SPAWN => {
                    let result = Proto::parse_tct_weapon_spawn(&message);

                    if result.is_err() {
                        log(result.unwrap_err());
                    }

                    log("Weapon spawned");

                    let (weapon_type, x, y) = result.unwrap();
                    
                    let new_weapon = Weapon::new(weapon_type, x, y);

                    self.ground_weapons.push(new_weapon);
                },
                Proto::TCT_REMOVE_WEAPON => {
                    let weapon_i = Proto::parse_tct_remove_weapon(&message);

                    if weapon_i.is_err() {
                        continue;
                    }

                    self.ground_weapons.remove(weapon_i.unwrap());
                },
                Proto::TCT_ASSIGN_WEAPON => {
                    let result = Proto::parse_tct_assign_weapon(&message);

                    if result.is_err() {
                        continue;
                    }

                    let (player_i, weapon_type) = result.unwrap();

                    self.players[player_i].assign_weapon(weapon_type);
                },
                Proto::TCT_NEW_PROJECTILE => {
                    let result = Proto::parse_tct_new_projectile(&message);

                    if result.is_err() {
                        continue;
                    }

                    let (projectile_type, x, y, vel_x, vel_y) = result.unwrap();

                    if self.audio.is_some() { 
                        let audio = self.audio.as_ref().unwrap();
                        let sound = match projectile_type {
                            Projectile::TYPE_GRENADE => Sounds::GLAUNCHER,
                            Projectile::TYPE_BULLET | _ => Sounds::MINIGUN,
                        };
                        let sound_x = x - self.players[self.player_i.unwrap()].x;
                        let sound_y = y - self.players[self.player_i.unwrap()].y;

                        audio.play(sound, sound_x, sound_y).await;
                    };


                    let new_projectile = Projectile {
                        projectile_type,
                        x,
                        y,
                        vel_x,
                        vel_y,
                        // Only the server needs to track projectile owners
                        owner: 0
                    };

                    self.projectiles.push(new_projectile);
                },
                Proto::TCT_PROJECTILE_EXPLOSION => {
                    let projectile = Proto::parse_tct_projectile_explosion(&message);

                    if projectile.is_err() {
                        continue;
                    }

                    let projectile = projectile.unwrap();

                    if self.audio.is_some() { 
                        let audio = self.audio.as_ref().unwrap();
                        let sound = match projectile.projectile_type {
                            Projectile::TYPE_GRENADE => Sounds::GRENADE,
                            Projectile::TYPE_BULLET | _ => Sounds::BULLET,
                        };
                        let sound_x = projectile.x - self.players[self.player_i.unwrap()].x;
                        let sound_y = projectile.y - self.players[self.player_i.unwrap()].y;

                        audio.play(sound, sound_x, sound_y).await;
                    };

                    for pixel in projectile.draw_explosion() {
                        let (pixel_x, pixel_y) = pixel;
                        self.graphics.clear_map_pixel(pixel_x, pixel_y);
                    }

                    let damage_radius
                        = Projectile::PROJECTILE_TYPES[projectile.projectile_type].damage_radius;

                    self.flashes.push(Flash::new(projectile.x, projectile.y, damage_radius));
                },
                Proto::TCT_DESTROY_PROJECTILE => {
                    let projectile_i = Proto::parse_tct_destroy_projectile(&message);

                    if projectile_i.is_err() {
                        continue;
                    }

                    self.projectiles.remove(projectile_i.unwrap());
                }
                Proto::TCT_UPDATE_HEALTH => {
                    let new_health = Proto::parse_tct_update_health(&message);

                    if new_health.is_err() {
                        continue;
                    }

                    self.players[self.player_i.unwrap()].health = new_health.unwrap();
                },
                Proto::TCT_REMOVE_AMMO => {
                    self.players[self.player_i.unwrap()].ammo -= 1;
                },
                Proto::TCT_KILL_PLAYER => {
                    let killed_player_i = Proto::parse_tct_kill_player(&message);

                    if killed_player_i.is_err() {
                        continue;
                    }
                    log("Player killed");

                    self.players[killed_player_i.unwrap()].kill();
                },
                Proto::TCT_RESPAWN_PLAYER => {
                    let result = Proto::parse_tct_respawn_player(&message);

                    if result.is_err() {
                        continue;
                    }

                    let (respawn_player_i, x, y) = result.unwrap();

                    self.players[respawn_player_i].respawn(x, y);
                },
                Proto::TCT_GAME_OVER_STATS => {
                    let stats = Proto::parse_tct_game_over_stats(&message);

                    for i in 0..stats.len() {
                        let (kills, deaths) = stats[i];
                        self.players[i].kills = kills;
                        self.players[i].deaths = deaths;
                    }

                    self.state = States::GameOver;
                    self.state_changed = true;

                    self.connection.as_mut().unwrap().disconnect();

                    break 'game_loop;
                }

                _ => ()
            }
        }

        let this_player = &mut self.players[self.player_i.unwrap()];

        if this_player.alive {
            if self.input.is_down('A' as u32) {
                this_player.x -= Player::MOVE_SPEED*time_elapsed;
            } else if self.input.is_down('D' as u32) {
                this_player.x += Player::MOVE_SPEED*time_elapsed;
            }

            if self.input.is_down('W' as u32) {
                if self.jump_debounce == false && this_player.jumps > 0 {
                    this_player.vel_y = Player::JUMP_VEL;
                    this_player.jumps -= 1;
                    self.jump_debounce = true;
                }
            } else {
                self.jump_debounce = false;
            }

            this_player.angle = (((Graphics::GAME_CANVAS_HEIGHT/2) - mouse_coord_y) as f32)
                .atan2(((Graphics::GAME_CANVAS_WIDTH/2) - mouse_coord_x) as f32);

            VelSystem::update_player(this_player, time_elapsed);
            CollisionSystem::update_player(this_player, &self.graphics.map_context);

            if (this_player.x-this_player.x_last).abs() > 1.0
                || (this_player.y-this_player.y_last).abs() > 1.0
            {
                    let new_pos_message = Proto::tst_new_pos(
                        this_player.x.round(),
                        this_player.y.round());
                    self.connection.as_mut().unwrap().send(new_pos_message);

                    this_player.x_last = this_player.x;
                    this_player.y_last = this_player.y;
            }

            if self.input.is_down('E' as u32) {
                let take_weapon_message = Proto::tst_take_weapon();
                self.connection.as_mut().unwrap().send(take_weapon_message);
            }

            let (mouse_state_changed, new_state) = self.input.mouse_state_changed();
            if mouse_state_changed {
                let trigger_message = match new_state {
                    true => Proto::tst_trigger_pulled(),
                    false => Proto::tst_trigger_released(),
                };

                self.connection.as_mut().unwrap().send(trigger_message);
            }

            self.next_angle_update -= time_elapsed;
            if self.next_angle_update < 0.0 {
                let new_angle_message = Proto::tst_new_angle(this_player.angle);
                self.connection.as_mut().unwrap().send(new_angle_message);

                self.next_angle_update = ANGLE_UPDATE_PERIOD;
            }
        } else {
            self.players[self.player_i.unwrap()].time_to_respawn -= time_elapsed as i32;
        }

        for projectile in &mut self.projectiles {
            VelSystem::update_projectile(projectile, time_elapsed);
        }

        for player_i in 0..self.players.len() {
            // Don't interpolate this player's location
            if player_i == self.player_i.unwrap() {
                continue;
            }

            InterpSystem::update_player(&mut self.players[player_i], time_elapsed);
        }

        let mut flashes_removed: usize = 0;
        for flash_i in 0..self.flashes.len() {
            let flash_i = flash_i - flashes_removed;

            self.flashes[flash_i].ttl -= time_elapsed;

            if self.flashes[flash_i].ttl < 0.0 {
                self.flashes.remove(flash_i);
                flashes_removed += 1;
            }
        }

        self.graphics.render_game(
            &self.players,
            self.player_i.unwrap(),
            &self.ground_weapons,
            &self.projectiles,
            mouse_coord_x,
            mouse_coord_y,
            &self.flashes,
        );
    }

    pub fn game_over(&mut self) {
        if self.state_changed {
            self.players.sort_by_key(|player| core::cmp::Reverse(player.kills));

            self.graphics.update_canvas(States::GameOver);
            self.graphics.first_render_game_over(&self.players);
            self.state_changed = false;
        }

        let (_, y) = self.input.mouse_coordinates();
        let mouse_clicked = self.input.mouse_clicked();

        if y > self.graphics.height_divided*10 && y < self.graphics.height_divided*11 {
            if mouse_clicked && self.audio.is_none() {
                self.state = States::Unmatched;
                self.state_changed = true;
            }
        }
    }
}
