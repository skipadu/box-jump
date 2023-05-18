use bevy::{prelude::*, window::WindowResolution};
use bevy_rapier2d::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 30.0);
const PLAYER_COLOR: Color = Color::rgb(0.1, 0.8, 0.3);
const PLAYER_SPEED: f32 = 100.0;

enum ObstacleSize {
    Small,
    Large,
}

impl ObstacleSize {
    fn size(&self) -> Vec2 {
        match self {
            ObstacleSize::Small => Vec2::new(10.0, 10.0),
            ObstacleSize::Large => Vec2::new(30.0, 20.0),
        }
    }
}

const GROUND_SIZE: Vec2 = Vec2::new(900.0, 50.0);
const GROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const WALL_SIZE: Vec2 = Vec2::new(50.0, 400.0);

struct WallSpawnPositions {
    x_position_start: f32,
    x_position_finish: f32,
    y_position: f32,
}

impl WallSpawnPositions {
    fn new() -> Self {
        let x_position_start = WALL_SIZE.x / 2.0;
        let x_position_finish = GROUND_SIZE.x - WALL_SIZE.x / 2.0;
        let y_position = WALL_SIZE.y / 2.0;
        Self {
            x_position_start,
            x_position_finish,
            y_position,
        }
    }

    fn start(&self) -> Vec3 {
        return Vec3::new(self.x_position_start, self.y_position, 0.0);
    }

    fn finish(&self) -> Vec3 {
        return Vec3::new(self.x_position_finish, self.y_position, 0.0);
    }
}

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collidable;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Coin;

#[derive(Component)]
struct Ground;

// TODO: walls
#[derive(Component)]
struct Wall;

#[derive(Bundle)]
struct ObstacleBundle {
    sprite_bundle: SpriteBundle,
    obstacle: Obstacle,
    collidable: Collidable,
    rigid_body: RigidBody,
    collider: Collider,
}

impl ObstacleBundle {
    fn new(position: &Vec2, size: &ObstacleSize) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y + (size.size().y / 2.0), 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.2, 0.1),
                    custom_size: Some(size.size()),
                    ..default()
                },
                ..default()
            },
            obstacle: Obstacle,
            collidable: Collidable,
            rigid_body: RigidBody::Fixed,
            collider: Collider::cuboid(size.size().x / 2.0, size.size().y / 2.0),
        }
    }
}

#[derive(Bundle)]
struct CoinBundle {
    sprite_bundle: SpriteBundle,
    coin: Coin,
    collidable: Collidable,
    collider: Collider,
}

impl CoinBundle {
    fn new(position: Vec2) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: Vec3::new(8.0, 8.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.7, 0.4),
                    ..default()
                },
                // visibility: Visibility::Hidden,
                ..default()
            },
            coin: Coin,
            collidable: Collidable,
            collider: Collider::ball(1.0),
        }
    }
}

#[derive(Resource)]
struct GameScore {
    score: u32,
}

#[derive(Component)]
struct ScoreText;

#[derive(Resource)]
struct PlayerState {
    is_in_air: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(600.0, 400.),
                title: "Box Jump".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameScore { score: 0 })
        .add_startup_system(setup_system)
        .add_startup_system(setup_level_system)
        .add_systems(
            (
                check_coin_collision_system,
                check_ground_collision_system,
                player_movement_system.before(check_coin_collision_system),
                show_score_system.after(check_coin_collision_system),
                player_camera_system,
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .insert_resource(PlayerState { is_in_air: false })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .run();
}

fn get_wall_spawn_positions() -> WallSpawnPositions {
    let x_position_start = WALL_SIZE.x / 2.0;
    let x_position_finish = GROUND_SIZE.x - WALL_SIZE.x / 2.0;
    let y_position = WALL_SIZE.y / 2.0;
    return WallSpawnPositions {
        x_position_start,
        x_position_finish,
        y_position,
    };
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Draw player, a green box
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(100.0, 30.0, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            ..default()
        },
        RigidBody::Dynamic,
        Velocity::zero(),
        Collider::cuboid(PLAYER_SIZE.x / 2.0, PLAYER_SIZE.y / 2.0),
        Player,
        ActiveEvents::COLLISION_EVENTS,
        // AdditionalMassProperties::Mass(0.2),
        // LockedAxes::ROTATION_LOCKED,
    ));

    // Create ground
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(GROUND_SIZE.x / 2.0, -GROUND_SIZE.y / 2.0, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: GROUND_COLOR,
                custom_size: Some(GROUND_SIZE),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(GROUND_SIZE.x / 2.0, GROUND_SIZE.y / 2.0),
        Ground,
    ));

    // let wall_spawn_position = get_wall_spawn_positions();
    let wall_spawn_position = WallSpawnPositions::new();

    // Start wall
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: wall_spawn_position.start(),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(0.9, 0.5, 0.2),
                custom_size: Some(WALL_SIZE),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(WALL_SIZE.x / 2.0, WALL_SIZE.y / 2.0),
        Wall,
    ));
    // End wall
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: wall_spawn_position.finish(),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(0.3, 0.5, 0.3),
                custom_size: Some(WALL_SIZE),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(WALL_SIZE.x / 2.0, WALL_SIZE.y / 2.0),
        Wall,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/Hack-Regular.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/Hack-Regular.ttf"),
                font_size: 32.0,
                color: Color::WHITE,
            }),
        ]),
        ScoreText,
    ));
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut rb_velocities: Query<&mut Velocity, With<Player>>,
    player_state: Res<PlayerState>,
) {
    let mut velocity = rb_velocities.single_mut();
    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        velocity.linvel = Vec2::new(-PLAYER_SPEED, velocity.linvel.y);
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        velocity.linvel = Vec2::new(PLAYER_SPEED, velocity.linvel.y);
    }
    if keyboard_input.pressed(KeyCode::Space) && !player_state.is_in_air {
        velocity.linvel = Vec2::new(velocity.linvel.x, 100.0);
    }
}

fn check_coin_collision_system(
    coin_query: Query<Entity, With<Coin>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                for coin_entity in coin_query.iter() {
                    if a == &coin_entity || b == &coin_entity {
                        println!("Coin picked!");
                        commands.entity(coin_entity).despawn();
                        game_score.score += 1;
                    }
                }
            }
            CollisionEvent::Stopped(_a, _b, _) => {
                // Do nothing
            }
        }
    }
}

// // TODO: Currently obstacles are just "air"
// fn check_obstacle_collision_system() {}

fn check_ground_collision_system(
    player_query: Query<Entity, With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_state: ResMut<PlayerState>,
) {
    let player_entity = player_query.single();
    let ground_entity = ground_query.single();

    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                if (a == &ground_entity && b == &player_entity)
                    || (a == &player_entity && b == &ground_entity)
                {
                    println!("Player is grounded");
                    player_state.is_in_air = false;
                }
            }
            CollisionEvent::Stopped(a, b, _) => {
                if (a == &ground_entity && b == &player_entity)
                    || (a == &player_entity && b == &ground_entity)
                {
                    println!("Player is in air!");
                    player_state.is_in_air = true;
                }
            }
        }
    }
}

fn spawn_obstacle_and_coin(commands: &mut Commands, position: Vec2, size: ObstacleSize) {
    commands.spawn(ObstacleBundle::new(&position, &size));
    let coin_y_position = position.y + &size.size().y + 20.0;
    commands.spawn(CoinBundle::new(Vec2::new(position.x, coin_y_position)));
}

fn setup_level_system(mut commands: Commands) {
    spawn_obstacle_and_coin(&mut commands, Vec2::new(150.0, 0.0), ObstacleSize::Small);
    spawn_obstacle_and_coin(&mut commands, Vec2::new(320.0, 0.0), ObstacleSize::Large);
}

fn show_score_system(game_score: ResMut<GameScore>, mut query: Query<&mut Text, With<ScoreText>>) {
    for mut text in &mut query {
        text.sections[1].value = format!("{}", game_score.score);
    }
}

fn player_camera_system(
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    mut camera: Query<(&mut Camera, &mut Transform), Without<Player>>,
) {
    let (_entity, player_transform) = player_query.single_mut();
    let (_camera, mut camera_transform) = camera.single_mut();
    camera_transform.translation = player_transform.translation;
}
