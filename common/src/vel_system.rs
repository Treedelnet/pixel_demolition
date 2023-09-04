#[cfg(target_family = "wasm")]
use crate::player::Player;

use crate::projectile::Projectile;

pub struct VelSystem {}

impl VelSystem {
    pub const GRAVITY:f32 = 1.2/1000.0;

    #[cfg(target_family = "wasm")]
    pub fn update_player(player: &mut Player, time_elapsed: f32) {
        // Apply gravity to the average change in velocity
        player.vel_y += Self::GRAVITY*time_elapsed/2.0;
        player.y += player.vel_y * time_elapsed;
        player.vel_y += Self::GRAVITY*time_elapsed/2.0;
    }

    pub fn update_projectile(projectile: &mut Projectile, time_elapsed: f32) {
        projectile.x = projectile.x + projectile.vel_x * time_elapsed;

        // Apply gravity to the average change in velocity
        projectile.vel_y += Self::GRAVITY*time_elapsed/2.0;
        projectile.y += projectile.vel_y * time_elapsed;
        projectile.vel_y += Self::GRAVITY*time_elapsed/2.0;
    }
}
