use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct Sprites {
    pub map_atlas: Handle<TextureAtlas>,
    pub level: Handle<Image>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Sprites>()
            .add_systems(Startup, load_assets);
    }
}

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut sprites: ResMut<Sprites>,
) {
    let texture_atlas_handle = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("cavesofgallet_tiles.png"),
        Vec2::new(8.0, 8.0),
        8,
        12,
        None,
        None,
    ));
    sprites.map_atlas = texture_atlas_handle;
}
