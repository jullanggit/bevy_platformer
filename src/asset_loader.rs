use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, Debug, Default, AssetCollection)]
pub struct Sprites {
    #[asset(texture_atlas_layout(
        tile_size_x = 8.,
        tile_size_y = 8.,
        columns = 8,
        rows = 12,
        padding_x = 0.,
        padding_y = 0.,
        offset_x = 0.,
        offset_y = 0.
    ))]
    pub map_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "cavesofgallet_tiles.png")]
    pub map_texture: Handle<Image>,
    #[asset(path = "map1.png")]
    pub level1: Handle<Image>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Sprites>()
            .init_state::<SpritesLoadingStates>()
            .add_loading_state(
                LoadingState::new(SpritesLoadingStates::Loading)
                    .continue_to_state(SpritesLoadingStates::Finished)
                    .load_collection::<Sprites>(),
            )
            .add_systems(Startup, load_assets);
    }
}

#[derive(Clone, Eq, PartialEq, Default, States, Hash, Debug)]
pub enum SpritesLoadingStates {
    #[default]
    Loading,
    Finished,
}

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut sprites: ResMut<Sprites>,
) {
    // texture
    let texture_handle = asset_server.load("cavesofgallet_tiles.png");
    sprites.map_texture = texture_handle;

    // layout
    let layout = TextureAtlasLayout::from_grid(Vec2::new(8.0, 8.0), 8, 12, None, None);
    let layout_handle = texture_atlases.add(layout);
    sprites.map_layout = layout_handle;
}
