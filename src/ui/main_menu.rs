use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    LoadCharacter,
    CreateCharacter,
    ConnectToServer,
    InGame,
}

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub enum MenuButton {
    LoadCharacter,
    CreateCharacter,
    ConnectServer,
    Quit,
}

#[derive(Component)]
pub struct AnimatedTorch {
    timer: Timer,
    frame_count: usize,
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

pub fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Background Image
    commands.spawn((
        Sprite {
            image: asset_server.load("Stontex.png"),
            custom_size: Some(Vec2::new(1200., 800.)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., 0., -1.)),
        MainMenuUI,
    ));

    // Load torch sprite sheet
    let torch_texture = asset_server.load("animated_torch.png");

    // Create texture atlas layout - 9 frames in a row
    // Adjust columns/rows based on how the sprite sheet is laid out
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 64), 9, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Left Torch
    commands.spawn((
        Sprite::from_atlas_image(
            torch_texture.clone(),
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(-300.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
        AnimatedTorch {
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            frame_count: 9,
        },
        MainMenuUI,
    ));

    // Right Torch
    commands.spawn((
        Sprite::from_atlas_image(
            torch_texture.clone(),
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(300.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
        AnimatedTorch {
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            frame_count: 9, // Adjust based on your sprite sheet
        },
        MainMenuUI,
    ));

    // Root container
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
            // Title
            parent.spawn((
                Text::new("Alembic"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Multi-User Dungeon Client"),
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

            // Button container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    create_menu_button(parent, "Load Character", MenuButton::LoadCharacter);
                    create_menu_button(parent, "Create New Character", MenuButton::CreateCharacter);
                    create_menu_button(parent, "Connect to Server", MenuButton::ConnectServer);
                    create_menu_button(parent, "Quit", MenuButton::Quit);
                });
        });
}

pub fn create_menu_button(parent: &mut ChildBuilder, text: &str, button_type: MenuButton) {
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
                Text::new(text),
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
                    MenuButton::LoadCharacter => {
                        println!("Load Character selected");
                        next_state.set(GameState::LoadCharacter);
                    }
                    MenuButton::CreateCharacter => {
                        println!("Create Character selected");
                        next_state.set(GameState::CreateCharacter);
                    }
                    MenuButton::ConnectServer => {
                        println!("Connect to Server selected");
                        next_state.set(GameState::ConnectToServer);
                    }
                    MenuButton::Quit => {
                        exit.send(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
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
