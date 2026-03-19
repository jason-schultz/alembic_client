use bevy::prelude::*;

pub mod character_select;
pub mod connecting;
pub mod create_character;
pub mod downloading_assets;
pub mod in_game;
pub mod main_menu;
pub mod server_list;
pub mod world_select;

// Shared spinner component — used by any screen with an animated loading indicator.
#[derive(Component)]
pub struct Spinner {
    pub timer: Timer,
    pub frame: usize,
}

pub fn animate_spinner(time: Res<Time>, mut query: Query<(&mut Spinner, &mut Text)>) {
    let frames = ["|", "/", "—", "\\"];
    for (mut spinner, mut text) in &mut query {
        spinner.timer.tick(time.delta());
        if spinner.timer.just_finished() {
            spinner.frame = (spinner.frame + 1) % frames.len();
            text.0 = frames[spinner.frame].to_string();
        }
    }
}
