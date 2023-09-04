use pixel_demolition_common::player::Player;
use pixel_demolition_common::server_tick::ServerTick;

pub struct InterpSystem {}

impl InterpSystem {
    pub fn update_player(player: &mut Player, time_elapsed: f32) {
        let interp_count = (ServerTick::SERVER_TICK as f32)/time_elapsed;

        // If the player is moving from left to right
        if player.x_last < player.x_new {
            // If the player player's current position is still left of the new position
            // interpolate it
            if player.x < player.x_new {
                let total_distance = player.x_new - player.x_last;
                let interp_x = total_distance/interp_count;
                player.x += interp_x;
            }
        } else if player.x > player.x_new {
            if player.x > player.x_new {
                let total_distance = player.x_last - player.x_new;
                let interp_x = total_distance/interp_count;
                player.x -= interp_x;
            }
        }

        // If the player is moving from top to bottom
        if player.y_last < player.y_new {
            // If the player player's current position is still above the new position
            // interpolate it
            if player.y < player.y_new {
                let total_distance = player.y_new - player.y_last;
                let interp_y = total_distance/interp_count;
                player.y += interp_y;
            }
        } else if player.y > player.y_new {
            if player.y > player.y_new {
                let total_distance = player.y_last - player.y_new;
                let interp_y = total_distance/interp_count;
                player.y -= interp_y;
            }
        }
    }
}
