#[cfg(target_family = "wasm")]
use web_sys::CanvasRenderingContext2d;

use crate::player::Player;

#[cfg(not(target_family = "wasm"))]
use crate::projectile::Projectile;
#[cfg(not(target_family = "wasm"))]
use image::{self, RgbaImage};

pub struct CollisionSystem {}

impl CollisionSystem {
    #[cfg(target_family = "wasm")]
    pub fn update_player(player: &mut Player, map_context: &CanvasRenderingContext2d) {
        const PLAYER_WIDTH:f32 = Player::PLAYER_WIDTH as f32;
        const PLAYER_HEIGHT:f32 = Player::PLAYER_HEIGHT as f32;

        let vertical_cb_width = (PLAYER_WIDTH*3.0/4.0).round();
        let vertical_cb_start_x = (player.x-(vertical_cb_width/2.0)).round();
        let vertical_cb_start_y = (player.y-(PLAYER_HEIGHT/2.0)).round();

        let horizontal_cb_height = (PLAYER_HEIGHT*2.0/4.0).round();
        let horizontal_cb_start_x = (player.x-(PLAYER_WIDTH/2.0)).round();
        let horizontal_cb_start_y = (player.y-(horizontal_cb_height/2.0)).round();

        let top_coll_data = map_context.get_image_data(
            vertical_cb_start_x as f64,
            vertical_cb_start_y as f64,
            vertical_cb_width as f64,
            (PLAYER_HEIGHT/2.0).round() as f64,
        ).expect("Unable to get map data").data();

        let bottom_coll_data = map_context.get_image_data(
            vertical_cb_start_x as f64,
            player.y.round() as f64,
            vertical_cb_width as f64,
            (PLAYER_HEIGHT/2.0).round() as f64,
        ).expect("Unable to get map data").data();

        let left_coll_data = map_context.get_image_data(
            horizontal_cb_start_x as f64,
            horizontal_cb_start_y as f64,
            (PLAYER_WIDTH/2.0).round() as f64,
            horizontal_cb_height as f64,
        ).expect("Unable to get map data").data();

        let right_coll_data = map_context.get_image_data(
            player.x.round() as f64,
            horizontal_cb_start_y as f64,
            (PLAYER_WIDTH/2.0).round() as f64,
            horizontal_cb_height as f64
        ).expect("Unable to get map data").data();

        const COLLIDE_BOTTOM:u8=0;
        const COLLIDE_TOP:u8=1;
        const COLLIDE_LEFT:u8=2;
        const COLLIDE_RIGHT:u8=3;

        for bound in [COLLIDE_RIGHT, COLLIDE_LEFT, COLLIDE_BOTTOM, COLLIDE_TOP] {
            if bound == COLLIDE_BOTTOM {
                for i in (3..bottom_coll_data.len()).step_by(4) {
                    if bottom_coll_data[i as usize] > 0 {
                        let line_from_top = ((i as f32)/4.0/vertical_cb_width).round() as usize;
                        let line_from_bottom = ((PLAYER_HEIGHT/2.0) as usize)-line_from_top;
                        player.y -= line_from_bottom as f32;
                        player.y = player.y.round();
                        player.vel_y = 0.0;
                        player.jumps = 3;
                        break;
                    }
                }
            } else if bound == COLLIDE_TOP {
                for i in (3..top_coll_data.len()).step_by(4).rev() {
                    if top_coll_data[i as usize] > 0 {
                        let line_from_top = ((i as f32)/4.0/vertical_cb_width).round() as usize;
                        player.y += line_from_top as f32;
                        player.y = player.y.round();
                        player.vel_y = 0.0;
                        break;
                    }
                }
            } else if bound == COLLIDE_LEFT {
                // Iterate over pixel columns left to right
                'outer: for i in (0..((PLAYER_WIDTH/2.0).round() as usize)).rev() {
                    for j in 0..(horizontal_cb_height as usize) {
                        let alpha_i = (j*((PLAYER_WIDTH/2.0).round() as usize) + i)*4+3;
                        if left_coll_data[alpha_i] > 0 {
                            player.x += (i+1) as f32;
                            player.x = player.x.round();
                            break 'outer;
                        }
                    }
                }
            } else if bound == COLLIDE_RIGHT {
                // Iterate over pixel columns left to right
                'outer: for i in 0..((PLAYER_WIDTH/2.0).round() as usize) {
                    for j in 0..(horizontal_cb_height as usize) {
                        let alpha_i = (j*((PLAYER_WIDTH/2.0).round() as usize) + i)*4+3;
                        if right_coll_data[alpha_i] > 0 {
                            let column_from_left = (PLAYER_WIDTH/2.0).round()-(i as f32);
                            player.x -= column_from_left;
                            player.x = player.x.round();
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn projectile_collide_map(projectile: &Projectile, map: &mut RgbaImage) -> bool {
        if map.get_pixel(projectile.x as u32, projectile.y as u32)[3] > 0 {
            return true;
        }

        return false;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn point_collide_player(point_x: f32, point_y: f32, player: &Player) -> bool {
        if point_x < player.x + (Player::PLAYER_WIDTH/2) as f32
            && point_x > player.x - (Player::PLAYER_WIDTH/2) as f32 
            && point_y < player.y + (Player::PLAYER_HEIGHT/2) as f32 
            && point_y > player.y - (Player::PLAYER_HEIGHT/2) as f32
        {
            return true;
        }

        return false;
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn projectile_oob(projectile: &Projectile, map: &RgbaImage) -> bool {
        return Self::oob(projectile.x, projectile.y, map);
    }


    #[cfg(not(target_family = "wasm"))]
    pub fn player_oob(player: &Player, map: &RgbaImage) -> bool {
        return Self::oob(player.x, player.y, map);
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn oob(x: f32, y: f32, map: &RgbaImage) -> bool {
        if x < 2.0 || x >= (map.width() - 2) as f32 {
            return true;
        }

        if y < 2.0 || y >= (map.height() - 2) as f32 {
            return true;
        }

        return false;
    }


}
