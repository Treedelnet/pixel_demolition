use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

use pixel_demolition_common::weapon::Weapon;
use pixel_demolition_common::player::Player;
use pixel_demolition_common::projectile::Projectile;

use crate::flash::Flash;
use crate::states::States;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct Graphics {
    pub document: Document,
    canvas: HtmlCanvasElement,
    pub context: web_sys::CanvasRenderingContext2d,
    pub map: HtmlImageElement,
    pub map_canvas: HtmlCanvasElement,
    pub map_context: CanvasRenderingContext2d,
    pub bots: Vec<HtmlCanvasElement>,
    pub bot_contexts: Vec<CanvasRenderingContext2d>,
    pub dead_bot: HtmlImageElement,
    background: HtmlImageElement,
    weapons: Vec<HtmlImageElement>,
    projectile_icons: Vec<HtmlImageElement>,
    projectiles: Vec<HtmlImageElement>,
    reticle: HtmlImageElement,
    health: HtmlImageElement,
    width: i32,
    pub height: i32,
    pub height_divided: i32,
}

impl Graphics {
    const START_HEIGHT_SECTIONS: i32 = 12;

    const RADIAN_TOP: f32 = -std::f32::consts::PI / 2.0;
    const RADIAN_BOTTOM: f32 = std::f32::consts::PI / 2.0;

    const LOBBY_CANVAS_HEIGHT: i32 = 600;
    const LOBBY_CANVS_WIDTH: i32 = 800;

    pub const GAME_CANVAS_WIDTH: i32 = 512;
    pub const GAME_CANVAS_HEIGHT: i32 = 320;

    const PARALLAX_DIVIDER: f32 = 3.0;

    pub fn new(window: &web_sys::Window) -> Graphics {
        let document = window.document().unwrap();

        let canvas: HtmlCanvasElement = document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();

        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        context.set_image_smoothing_enabled(false);

        let background: HtmlImageElement = document
            .get_element_by_id("background")
            .unwrap()
            .dyn_into()
            .unwrap();

        let reticle: HtmlImageElement = document
            .get_element_by_id("reticle")
            .unwrap()
            .dyn_into()
            .unwrap();

        let health: HtmlImageElement = document
            .get_element_by_id("health")
            .unwrap()
            .dyn_into()
            .unwrap();

        let map: HtmlImageElement = document
            .get_element_by_id("map")
            .unwrap()
            .dyn_into()
            .unwrap();

        let map_canvas: HtmlCanvasElement = document
            .get_element_by_id("map_canvas")
            .unwrap()
            .dyn_into()
            .unwrap();

        let map_context: CanvasRenderingContext2d = map_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        map_canvas.set_width(map.width());
        map_canvas.set_height(map.height());

        let _ = map_context.draw_image_with_html_image_element(&map, 0.0, 0.0);

        let mut bots = Vec::new();
        let mut bot_contexts = Vec::new();

        let bot_base_image: HtmlImageElement = document
            .get_element_by_id("bot_base")
            .unwrap()
            .dyn_into()
            .unwrap();

        for color_i in 0..Player::COLOR_NAMES.len() {
            let color_name = Player::COLOR_NAMES[color_i];

            let bot_canvas_name = &format!("bot_{}", color_name.to_lowercase());

            let bot: HtmlCanvasElement = document
                .get_element_by_id(bot_canvas_name)
                .unwrap()
                .dyn_into()
                .unwrap();

            let bot_context: CanvasRenderingContext2d =
                bot.get_context("2d").unwrap().unwrap().dyn_into().unwrap();

            bot.set_width(bot_base_image.width());
            bot.set_height(bot_base_image.height());
            let _ = bot_context.draw_image_with_html_image_element(&bot_base_image, 0.0, 0.0);

            let mut image_bytes = bot_context
                .get_image_data(
                    0.0,
                    0.0,
                    bot_base_image.width() as f64,
                    bot_base_image.height() as f64,
                )
                .expect("Unable to get image data")
                .data();

            for i in (0..image_bytes.len()).step_by(4) {
                if image_bytes[i] == 255 {
                    image_bytes[i] = Player::COLORS[color_i][0];
                    image_bytes[i + 1] = Player::COLORS[color_i][1];
                    image_bytes[i + 2] = Player::COLORS[color_i][2];
                }
            }

            let _ = bot_context.put_image_data(
                &ImageData::new_with_u8_clamped_array(
                    wasm_bindgen::Clamped(&image_bytes),
                    bot_base_image.width(),
                )
                .unwrap(),
                0.0,
                0.0,
            );

            bot_contexts.push(bot_context);
            bots.push(bot);
        }

        let dead_bot: HtmlImageElement = document
            .get_element_by_id("dead_bot")
            .unwrap()
            .dyn_into()
            .unwrap();

        let mut weapons: Vec<HtmlImageElement> = Vec::new();

        for i in 0..Weapon::WEAPON_TYPES.len() {
            let weapon: HtmlImageElement = document
                .get_element_by_id(Weapon::WEAPON_TYPES[i].name)
                .unwrap()
                .dyn_into()
                .unwrap();

            weapons.push(weapon);
        }

        let mut projectile_icons: Vec<HtmlImageElement> = Vec::new();

        for i in 0..Projectile::PROJECTILE_TYPES.len() {
            let projectile_icon: HtmlImageElement = document
                .get_element_by_id(&format!("{}_icon", Projectile::PROJECTILE_TYPES[i].name))
                .unwrap()
                .dyn_into()
                .unwrap();

            projectile_icons.push(projectile_icon);
        }

        let mut projectiles: Vec<HtmlImageElement> = Vec::new();

        for i in 0..Projectile::PROJECTILE_TYPES.len() {
            let projectile: HtmlImageElement = document
                .get_element_by_id(Projectile::PROJECTILE_TYPES[i].name)
                .unwrap()
                .dyn_into()
                .unwrap();

            projectiles.push(projectile);
        }

        return Graphics {
            document,
            canvas,
            context,
            map,
            map_canvas,
            map_context,
            bots,
            bot_contexts,
            dead_bot,
            background,
            reticle,
            health,
            weapons,
            projectile_icons,
            projectiles,
            width: 0,
            height: 0,
            height_divided: 0,
        };
    }

    pub fn update_canvas(&mut self, state: States) {
        match state {
            States::Unmatched | States::Lobby | States::GameOver => {
                self.width = Self::LOBBY_CANVS_WIDTH;
                self.height = Self::LOBBY_CANVAS_HEIGHT;
                let _ = self.canvas.style().set_property("cursor", "auto");
            },
            States::Game => {
                self.width = Self::GAME_CANVAS_WIDTH;
                self.height = Self::GAME_CANVAS_HEIGHT;
                let _ = self.canvas.style().set_property("cursor", "none");
            }
        }

        self.canvas.set_width(self.width as u32);
        self.canvas.set_height(self.height as u32);

        self.height_divided = self.height / Self::START_HEIGHT_SECTIONS;
    }

    pub fn draw_text(&self, vert_height: i32, text: &String) {
        self.context
            .set_font(&format!("{}px monospace", self.height_divided / 2));

        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");

        self.context.set_fill_style(&"white".into());

        self.context
            .fill_text(
                &text,
                (self.width / 2) as f64,
                (self.height_divided * vert_height + self.height_divided / 2) as f64,
            )
            .unwrap();
    }

    pub fn draw_boxed_text(&self, vert_heigh: i32, text: &String) {
        self.context.set_fill_style(&"black".into());
        self.context.fill_rect(
            100.0,
            (self.height_divided * vert_heigh + (self.height_divided / 10)) as f64,
            (self.width - 200) as f64,
            (self.height_divided - (self.height_divided / 5)) as f64,
        );

        self.context.set_stroke_style(&"white".into());
        self.context.stroke_rect(
            100.0,
            (self.height_divided * vert_heigh + (self.height_divided / 10)) as f64,
            (self.width - 200) as f64,
            (self.height_divided - (self.height_divided / 5)) as f64,
        );

        self.draw_text(vert_heigh, &text);
    }

    pub fn first_render_unmatched(&mut self) {
        self.context.set_fill_style(&"black".into());
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        self.draw_text(1, &String::from("Your Name"));
        self.draw_boxed_text(2, &String::from(""));

        self.draw_text(4, &String::from("Enter Existing Game Code"));
        self.draw_boxed_text(5, &String::from(""));
        self.draw_boxed_text(6, &String::from("Join Existing Game"));
        self.draw_boxed_text(8, &String::from("Start New Game"));
        self.draw_boxed_text(10, &String::from("Enable Audio"));
    }

    pub fn update_render_unmatched_name_code(&mut self, name: &String, code: &String) {
        self.height_divided = self.height / Self::START_HEIGHT_SECTIONS;
        self.context.set_fill_style(&"black".into());
        self.draw_boxed_text(2, name);
        self.draw_boxed_text(5, code);
    }

    pub fn update_render_unmatched_audio_enabled(&mut self) {
        self.draw_boxed_text(10, &String::from("Audio Enabled"));
    }

    pub fn render_lobby(&mut self, code: &String, players: &Vec<Player>) {
        self.context.set_fill_style(&"black".into());
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        self.draw_text(1, &format!("Code: {}", code));

        self.draw_boxed_text(8, &String::from("Ready"));

        for i in 0..players.len() {
            let ready = match players[i].ready {
                true => "Ready",
                false => "Not Ready",
            };

            self.draw_text(
                (i as i32) + 3,
                &format!(
                    "{} - {} - {}",
                    players[i].name,
                    Player::COLOR_NAMES[i],
                    ready
                ),
            );
        }
    }

    pub fn clear_map_pixel(&self, x: i32, y: i32) {
        self.map_context.clear_rect(x as f64, y as f64, 1.0, 1.0)
    }

    pub fn render_game(
        &self,
        players: &Vec<Player>,
        this_player_i: usize,
        ground_weapons: &Vec<Weapon>,
        projectiles: &Vec<Projectile>,
        mouse_coord_x: i32,
        mouse_coord_y: i32,
        flashes: &Vec<Flash>,
    ) {
        let half_map_x = self.map.width() as f32 / 2.0;
        let half_map_y = self.map.height() as f32 / 2.0;

        // Find offset from center based on player location relative to map background
        let parallax_shifted_x = (half_map_x - players[this_player_i].x) / Self::PARALLAX_DIVIDER;
        let parallax_shifted_y = (half_map_y - players[this_player_i].y) / Self::PARALLAX_DIVIDER;

        let _ = self
            .context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.background,
                (players[this_player_i].x + parallax_shifted_x - (self.width as f32) / 2.0) as f64,
                (players[this_player_i].y + parallax_shifted_y - (self.height as f32) / 2.0) as f64,
                self.width as f64,
                self.height as f64,
                0.0,
                0.0,
                self.width as f64,
                self.height as f64,
            );

        let _ = self
            .context
            .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.map_canvas,
                (players[this_player_i].x - (self.width as f32) / 2.0).round() as f64,
                (players[this_player_i].y - (self.height as f32) / 2.0).round() as f64,
                self.width as f64,
                self.height as f64,
                0.0,
                0.0,
                self.width as f64,
                self.height as f64,
            );

        for i in 0..players.len() {
            self.context.save();

            if i == this_player_i as usize {
                self.context
                    .translate((self.width / 2) as f64, (self.height / 2) as f64)
                    .unwrap();
            } else {
                self.context
                    .translate(
                        players[i].x.round() as f64
                            - players[this_player_i as usize].x.round() as f64
                            + (self.width / 2) as f64,
                        players[i].y.round() as f64
                            - players[this_player_i as usize].y.round() as f64
                            + (self.height / 2) as f64,
                    )
                    .unwrap();
            }

            let facing_right = match players[i as usize].angle {
                ang if ang > Self::RADIAN_TOP && ang < Self::RADIAN_BOTTOM => false,
                _ => true,
            };

            self.context.save();
            match facing_right {
                false => self.context.scale(1.0, 1.0).expect("Unable to set scale"),
                true => self.context.scale(-1.0, 1.0).expect("Unable to set scale"),
            }

            if players[i].alive {
                self.context
                    .draw_image_with_html_canvas_element(
                        &self.bots[i as usize],
                        -((Player::PLAYER_WIDTH / 2) as f64),
                        -((Player::PLAYER_HEIGHT / 2) as f64),
                    )
                    .expect("Unable to draw sprite");
            } else {
                // Draw dead player only if there is ground below them
                let below_player_data = self
                    .map_context
                    .get_image_data(
                        players[i].x as f64,
                        (players[i].y + (Player::PLAYER_HEIGHT / 2) as f32) as f64,
                        2.0,
                        2.0,
                    )
                    .unwrap()
                    .data();

                for below_player_data_i in (3..below_player_data.len()).step_by(4) {
                    if below_player_data[below_player_data_i] > 0 {
                        self.context
                            .draw_image_with_html_image_element(
                                &self.dead_bot,
                                -((Player::PLAYER_WIDTH / 2) as f64),
                                -((Player::PLAYER_HEIGHT / 2) as f64),
                            )
                            .expect("Unable to draw sprite");
                        break;
                    }
                }
            }

            self.context.restore();

            if players[i].weapon_type.is_some() {
                self.context.rotate(players[i].angle as f64).unwrap();

                self.context.translate(-36.0, 0.0).unwrap();

                self.context
                    .draw_image_with_html_image_element(
                        &self.weapons[players[i].weapon_type.unwrap() as usize],
                        0.0,
                        -4.0,
                    )
                    .expect("Unable to draw weapon sprite");
            }
            self.context.restore();
        }

        for i in 0..ground_weapons.len() {
            let weapon_type = ground_weapons[i].weapon_type;

            self.context
                .draw_image_with_html_image_element(
                    &self.weapons[weapon_type as usize],
                    ground_weapons[i].x as f64 - players[this_player_i as usize].x.round() as f64
                        + (self.width / 2) as f64,
                    ground_weapons[i].y as f64 - players[this_player_i as usize].y.round() as f64
                        + (self.height / 2) as f64,
                )
                .expect("Unable to draw sprite");
        }

        for i in 0..projectiles.len() {
            let projectile_type = projectiles[i].projectile_type;

            self.context
                .draw_image_with_html_image_element(
                    &self.projectiles[projectile_type as usize],
                    projectiles[i].x as f64 - players[this_player_i as usize].x.round() as f64
                        + (self.width / 2) as f64,
                    projectiles[i].y as f64 - players[this_player_i as usize].y.round() as f64
                        + (self.height / 2) as f64,
                )
                .expect("Unable to draw sprite");
        }

        let reticle_x = mouse_coord_x - (self.reticle.width() / 2) as i32;
        let reticle_y = mouse_coord_y - (self.reticle.height() / 2) as i32;
        let _ = self.context.draw_image_with_html_image_element(
            &self.reticle,
            reticle_x as f64,
            reticle_y as f64,
        );

        for i in 0..(players[this_player_i].health.round() as usize) {
            let _ = self.context.draw_image_with_html_image_element(
                &self.health,
                6.0 + 20.0 * (i as f64),
                6.0,
            );
        }

        if players[this_player_i as usize].weapon_type.is_some() {
            for i in 0..(players[this_player_i as usize].ammo) {
                let weapon_type = players[this_player_i as usize].weapon_type.unwrap();
                let projectile_type = Weapon::WEAPON_TYPES[weapon_type].projectile_type;

                let ammo_spacing = match projectile_type {
                    Projectile::TYPE_BULLET => 4.0,
                    _ => 20.0,
                };

                let _ = self.context.draw_image_with_html_image_element(
                    &self.projectile_icons[projectile_type as usize],
                    6.0 + ammo_spacing * (i as f64),
                    28.0,
                );
            }
        }

        // If this player is dead, show the respawn timer
        if !players[this_player_i].alive {
            let time_to_respawn_ms = players[this_player_i].time_to_respawn;
            let time_to_respawn_s = (time_to_respawn_ms / 1000).to_string();

            self.context
                .set_font(&format!("{}px monospace", self.height / 2));

            self.context.set_text_align("center");
            self.context.set_text_baseline("middle");

            self.context.set_fill_style(&"black".into());

            self.context
                .fill_text(
                    &time_to_respawn_s,
                    (self.width / 2) as f64,
                    (self.height / 2) as f64,
                )
                .unwrap();
        }

        for flash in flashes {
            self.context.set_fill_style(&"yellow".into());

            let offset_x = flash.x - (players[this_player_i].x.round()) + (self.width / 2) as f32;
            let offset_y = flash.y - (players[this_player_i].y.round()) + (self.height / 2) as f32;

            self.context.begin_path();

            let _ = self.context.arc(
                offset_x as f64,
                offset_y as f64,
                flash.radius as f64,
                0.0,
                std::f64::consts::PI * 2.0,
            );

            self.context.fill();
        }
    }

    pub fn first_render_game_over(&mut self, players_sorted: &Vec<Player>) {
        self.context.set_fill_style(&"black".into());
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        self.draw_text(1, &format!("Name - Color - Kills - Deaths"));

        for player_i in 0..players_sorted.len() {
            let player = &players_sorted[player_i];

            let line = format!(
                "{} - {} - {} - {}",
                player.name,
                Player::COLOR_NAMES[player_i],
                player.kills,
                player.deaths,
            );

            self.draw_text((3 + player_i) as i32, &line);
        }

        self.draw_boxed_text(10, &String::from("Start New Game"));
    }
}
