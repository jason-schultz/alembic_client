use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    ServerList,
    Connecting,
    WorldSelect,
    DownloadingAssets,
    CharacterSelect,
    CreateCharacter,
    InGame,
}

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub enum MenuButton {
    Play,
    Quit,
}

#[derive(Component)]
pub struct AnimatedTorch {
    timer: Timer,
    frame_count: usize,
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(), Transform::from_xyz(0.0, 0.0, 0.0)));
}

pub fn reset_camera(mut query: Query<&mut Transform, With<Camera2d>>) {
    if let Ok(mut transform) = query.get_single_mut() {
        transform.translation = Vec3::ZERO;
    }
}

pub fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Full-screen background via UI ImageNode — scales with window
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode {
            image: asset_server.load("Stontex.png"),
            ..default()
        },
        ZIndex(-1),
        MainMenuUI,
    ));

    // Torches
    let torch_texture = asset_server.load("animated_torch.png");
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 64),
        9,
        1,
        None,
        None,
    ));

    for x in [-300.0f32, 300.0] {
        commands.spawn((
            Sprite::from_atlas_image(
                torch_texture.clone(),
                TextureAtlas {
                    layout: layout.clone(),
                    index: 0,
                },
            ),
            Transform::from_xyz(x, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
            AnimatedTorch {
                timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                frame_count: 9,
            },
            MainMenuUI,
        ));
    }

    // Root UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MainMenuUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Alembic"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("A Tabletop RPG Campaign Client"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_menu_button(parent, "Play", MenuButton::Play);
                    spawn_menu_button(parent, "Quit", MenuButton::Quit);
                });
        });
}

pub fn spawn_menu_button(parent: &mut ChildBuilder, label: &str, button_type: MenuButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(300.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            button_type,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

pub fn main_menu_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match button {
                    MenuButton::Play => next_state.set(GameState::ServerList),
                    MenuButton::Quit => {
                        exit.send(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        }
    }
}

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn animate_torches(time: Res<Time>, mut query: Query<(&mut AnimatedTorch, &mut Sprite)>) {
    for (mut torch, mut sprite) in &mut query {
        torch.timer.tick(time.delta());
        if torch.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % torch.frame_count;
            }
        }
    }
}
