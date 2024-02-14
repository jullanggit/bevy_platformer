use bevy::prelude::*;

// Plugin
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

// Systems
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
