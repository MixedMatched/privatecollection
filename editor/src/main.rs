#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use std::path::PathBuf;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<State>()
        .init_resource::<InFile>()
        .add_systems(Startup, setup)
        .add_plugins((menu::MenuPlugin, editor::EditorPlugin))
        .run();
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum State {
    #[default]
    StartMenu,
    Editor,
}

#[derive(Resource, Default)]
struct InFile(Option<PathBuf>);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

mod menu {
    use bevy::prelude::*;
    use rfd::FileDialog;

    use super::{despawn, InFile, State, TEXT_COLOR};

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum MenuState {
        #[default]
        Main,
        LoadMap,
    }

    pub struct MenuPlugin;

    impl Plugin for MenuPlugin {
        fn build(&self, app: &mut App) {
            app.add_state::<MenuState>()
                .add_systems(OnEnter(State::StartMenu), menu_setup)
                .add_systems(OnEnter(MenuState::Main), main_menu_setup)
                .add_systems(OnEnter(MenuState::LoadMap), load_map_setup)
                .add_systems(
                    Update,
                    (menu_action, button_system).run_if(in_state(State::StartMenu)),
                )
                .add_systems(OnExit(MenuState::Main), despawn::<OnMainMenu>)
                .add_systems(OnExit(MenuState::LoadMap), despawn::<OnLoadMap>)
                .add_systems(OnExit(State::StartMenu), despawn::<OnMainMenu>)
                .add_systems(OnExit(State::StartMenu), despawn::<OnLoadMap>);
        }
    }

    #[derive(Component)]
    struct OnMainMenu;

    #[derive(Component)]
    struct OnNewMap;

    #[derive(Component)]
    struct OnLoadMap;

    #[derive(Component)]
    enum MenuAction {
        BackToMainMenu,
        NewMap,
        LoadMap,
        FileSelect,
        Continue,
    }

    const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

    #[derive(Component)]
    struct SelectedComponent;

    fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Main);
    }

    fn button_system(
        mut interaction_query: Query<
            (
                &Interaction,
                &mut BackgroundColor,
                Option<&SelectedComponent>,
            ),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color, selected) in &mut interaction_query {
            *color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    fn main_menu_setup(mut commands: Commands) {
        // Common style for all buttons on the screen
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnMainMenu,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Display the game name
                        parent.spawn(
                            TextBundle::from_section(
                                "PC Map Editor",
                                TextStyle {
                                    font_size: 80.0,
                                    color: TEXT_COLOR,
                                    ..default()
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );

                        // Display three buttons for each action available from the main menu:
                        // - new game
                        // - settings
                        // - quit
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuAction::LoadMap,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Load Map",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuAction::NewMap,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "New Map",
                                    button_text_style.clone(),
                                ));
                            });
                    });
            });
    }

    fn load_map_setup(mut commands: Commands) {
        // Common style for all buttons on the screen
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnLoadMap,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuAction::FileSelect,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "File Select",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuAction::Continue,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Continue",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuAction::BackToMainMenu,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Back",
                                    button_text_style.clone(),
                                ));
                            });
                    });
            });
    }

    fn menu_action(
        interaction_query: Query<(&Interaction, &MenuAction), (Changed<Interaction>, With<Button>)>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<State>>,
        mut in_file: ResMut<InFile>,
    ) {
        for (interaction, action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match action {
                    MenuAction::BackToMainMenu => {
                        in_file.0 = None;
                        menu_state.set(MenuState::Main)
                    }
                    MenuAction::NewMap => game_state.set(State::Editor),
                    MenuAction::LoadMap => menu_state.set(MenuState::LoadMap),
                    MenuAction::FileSelect => {
                        let file = FileDialog::new().add_filter("map", &["map"]).pick_file();
                        in_file.0 = file;
                    }
                    MenuAction::Continue => {
                        if in_file.0.is_some() {
                            game_state.set(State::Editor);
                        }
                    }
                }
            }
        }
    }
}

mod editor {
    use std::{fs::File, io::Write};

    use super::{despawn, InFile, State, TEXT_COLOR};
    use bevy::{
        input::mouse::{MouseMotion, MouseWheel},
        prelude::*,
    };
    use map::{Map, Tile, TileType};
    use rfd::FileDialog;

    #[derive(Component)]
    struct TileComponent;

    pub struct EditorPlugin;

    #[derive(Resource, Default)]
    struct LiveMap(Map);

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum DrawState {
        #[default]
        Refresh,
        Update,
    }

    impl Plugin for EditorPlugin {
        fn build(&self, app: &mut App) {
            app.add_state::<DrawState>()
                .add_systems(OnEnter(State::Editor), editor_setup)
                .init_resource::<LiveMap>()
                .add_systems(
                    Update,
                    (mouse_navigation, mouse_input, keyboard_input)
                        .chain()
                        .run_if(in_state(State::Editor)),
                )
                .add_systems(
                    OnEnter(DrawState::Refresh),
                    (despawn::<TileComponent>, refresh_map).chain(),
                );
        }
    }

    fn editor_setup(
        mut in_file: ResMut<InFile>,
        mut map: ResMut<LiveMap>,
        mut draw_state: ResMut<NextState<DrawState>>,
    ) {
        let mut m = map::Map { tiles: Vec::new() };
        if let Some(file) = in_file.0.take() {
            let file = std::fs::File::open(file).unwrap();

            if let Ok(file_map) = serde_json::from_reader(file) {
                m = file_map;
            } else {
                println!("Failed to load map from file");
            }
        }

        map.0 = m;

        draw_state.set(DrawState::Refresh);
    }

    fn refresh_map(
        commands: Commands,
        map: Res<LiveMap>,
        mut draw_state: ResMut<NextState<DrawState>>,
    ) {
        render_map(commands, &map.0);

        draw_state.set(DrawState::Update);
    }

    fn render_map(mut commands: Commands, map: &map::Map) {
        for (i, row) in map.tiles.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                render_tile(&mut commands, j, i, tile);
            }
        }
    }

    fn render_tile(commands: &mut Commands, x: usize, y: usize, tile: &Tile) {
        commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: match tile.tile_type {
                            TileType::Walkable => Color::WHITE,
                            TileType::Blocked => Color::GRAY,
                        },
                        custom_size: Some(Vec2::new(32.0, 32.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * 32.0,
                        y as f32 * 32.0,
                        0.0,
                    )),
                    ..default()
                },
                TileComponent,
            ))
            .with_children(|parent| {
                let mut text = String::new();

                text += ("Object: ".to_owned()
                    + &if let Some(object) = tile.object {
                        object.object_type.to_string()
                    } else {
                        "None".to_string()
                    })
                    .as_str();

                text += "\n";

                text += ("FloorObject: ".to_owned()
                    + &if let Some(floor) = tile.floor_object {
                        floor.object_type.to_string()
                    } else {
                        "None".to_string()
                    })
                    .as_str();

                text += "\n";

                text += ("Connection: ".to_owned()
                    + &if let Some(connection) = tile.connection.clone() {
                        connection.to_string()
                    } else {
                        "None".to_string()
                    })
                    .as_str();

                parent.spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: text,
                            style: TextStyle {
                                font: Handle::default(),
                                font_size: 10.0,
                                color: TEXT_COLOR,
                            },
                        }],
                        alignment: TextAlignment::Center,
                        ..default()
                    },
                    ..default()
                });
            });
    }

    fn mouse_navigation(
        mut camera: Query<&mut Transform, With<Camera>>,
        mouse_input: Res<Input<MouseButton>>,
        mut cursor: EventReader<MouseMotion>,
        mut scroll: EventReader<MouseWheel>,
    ) {
        if let Ok(mut camera) = camera.get_single_mut() {
            if mouse_input.pressed(MouseButton::Right) {
                for event in cursor.iter() {
                    camera.translation.x -= event.delta.x * camera.scale.x * 0.2;
                    camera.translation.y += event.delta.y * camera.scale.y * 0.2;
                }
            }

            for event in scroll.iter() {
                camera.scale.x += event.y * camera.scale.x * 0.1;
                camera.scale.y += event.y * camera.scale.y * 0.1;
            }
        }
    }

    fn mouse_input(
        mut commands: Commands,
        mut camera: Query<&mut Transform, With<Camera>>,
        mut map: ResMut<LiveMap>,
        mouse_input: Res<Input<MouseButton>>,
        windows: Query<&Window>,
        mut draw_state: ResMut<NextState<DrawState>>,
    ) {
        if let Ok(mut camera) = camera.get_single_mut() {
            if let Ok(window) = windows.get_single() {
                if mouse_input.just_pressed(MouseButton::Left) {
                    let x = window.cursor_position().unwrap().x - window.width() * 0.5;
                    let y = window.height() * 0.5 - window.cursor_position().unwrap().y;

                    let x = x + camera.translation.x;
                    let y = y + camera.translation.y;

                    let x = x * camera.scale.x;
                    let y = y * camera.scale.y;

                    let x = x.floor() / 32.0;
                    let y = y.floor() / 32.0;

                    map.0.expand_to(x as i32, y as i32);

                    if let Some(tile) = map.0.tiles.get_mut(y as usize) {
                        if let Some(tile) = tile.get_mut(x as usize) {
                            tile.tile_type = TileType::Walkable;
                            render_tile(&mut commands, x as usize, y as usize, tile);
                        }
                    }

                    if x < 0.0 {
                        camera.translation.x = -((window.cursor_position().unwrap().x
                            - window.width() * 0.5)
                            * camera.scale.x);
                    }
                    if y < 0.0 {
                        camera.translation.y = -((window.height() * 0.5
                            - window.cursor_position().unwrap().y)
                            * camera.scale.y);
                    }

                    draw_state.set(DrawState::Refresh);
                }
            }
        }
    }

    fn keyboard_input(map: ResMut<LiveMap>, keyboard_input: Res<Input<KeyCode>>) {
        if keyboard_input.just_pressed(KeyCode::S) {
            let file_dialog = FileDialog::new().add_filter("Map", &["map"]).save_file();

            if let Some(path) = file_dialog {
                let mut file = File::create(path).unwrap();
                let mut map = map.0.clone();
                map.trim();
                map.pad(1);

                let map = serde_json::to_string(&map).unwrap();
                file.write_all(map.as_bytes()).unwrap();
            }
        }
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
