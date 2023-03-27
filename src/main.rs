use bevy::{prelude::*, sprite::collide_aabb::collide, window::WindowResolution};
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

const COIN_SIZE: Vec3 = Vec3::new(8.0, 8.0, 0.0);
const COIN_COLOR: Color = Color::rgb(0.8, 0.7, 0.4);

const GROUND_SIZE: Vec2 = Vec2::new(500.0, 50.0);
const GROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collidable;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Coin;

enum CollisionEventType {
    ObstacleCrash,
    CoinCollect,
}

#[derive(Bundle)]
struct ObstacleBundle {
    sprite_bundle: SpriteBundle,
    obstacle: Obstacle,
    collidable: Collidable,
}

impl ObstacleBundle {
    fn new(position: Vec2, size: ObstacleSize) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: Vec3::new(size.size().x, size.size().y, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.2, 0.1),
                    ..default()
                },
                ..default()
            },
            obstacle: Obstacle,
            collidable: Collidable,
        }
    }
}

#[derive(Bundle)]
struct CoinBundle {
    sprite_bundle: SpriteBundle,
    coin: Coin,
    collidable: Collidable,
}

impl CoinBundle {
    fn new(position: Vec2) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: COIN_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: COIN_COLOR,
                    ..default()
                },
                // visibility: Visibility::Hidden,
                ..default()
            },
            coin: Coin,
            collidable: Collidable,
        }
    }
}

struct CollisionEvent(CollisionEventType);

#[derive(Resource)]
struct GameScore {
    score: u32,
}

#[derive(Component)]
struct ScoreText;

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
        .add_event::<CollisionEvent>()
        .insert_resource(GameScore { score: 0 })
        .add_startup_system(setup_system)
        .add_startup_system(setup_level_system)
        .add_systems(
            (
                check_collision_system,
                player_movement_system.before(check_collision_system),
                play_collision_sound_system.after(check_collision_system),
                score_system.after(check_collision_system),
                show_score_system.after(check_collision_system),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .run();
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Draw player, a green box
    commands.spawn((
        SpriteBundle {
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
        ExternalImpulse::default(),
        // AdditionalMassProperties::Mass(0.2),
        // LockedAxes::ROTATION_LOCKED,
    ));

    // Create ground
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -GROUND_SIZE.y, 0.0),
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
    mut query: Query<&mut Transform, With<Player>>,
    mut ext_impulses: Query<&mut ExternalImpulse, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }
    let new_player_position = player_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;
    player_transform.translation.x = new_player_position;

    let mut ext_impulse = ext_impulses.single_mut();
    let mut force = Vec2::new(0.0, 0.0);
    let mut torque = 0.0;
    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        force.y += 1.0;
        // torque -= 0.5;
    }
    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        force.y -= 1.0;
        // torque += 0.5;
    }
    ext_impulse.impulse = force;
    ext_impulse.torque_impulse = torque;
}

fn check_collision_system(
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    collider_query: Query<(Entity, &Transform, Option<&Obstacle>), With<Collidable>>,
    coin_collider_query: Query<(Entity, &Transform, Option<&Coin>), With<Collidable>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut commands: Commands,
) {
    let (_entity, player_transform) = player_query.single_mut();
    let player_size = player_transform.scale.truncate();

    // Check collision with player vs obstacles
    for (_entity, transform, maybe_obstacle) in &collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(_c) = collision {
            if maybe_obstacle.is_some() {
                collision_events.send(CollisionEvent(CollisionEventType::ObstacleCrash));
            }
        }
    }
    // Check collision with player vs coins
    for (entity, transform, maybe_coin) in &coin_collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(_c) = collision {
            if maybe_coin.is_some() {
                collision_events.send(CollisionEvent(CollisionEventType::CoinCollect));
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_obstacle_and_coin(commands: &mut Commands, position: Vec2, size: ObstacleSize) {
    commands.spawn(ObstacleBundle::new(position, size));

    commands.spawn(CoinBundle::new(Vec2::new(position.x, position.y + 20.0)));
}

fn setup_level_system(mut commands: Commands) {
    spawn_obstacle_and_coin(&mut commands, Vec2::new(50.0, 0.0), ObstacleSize::Small);
    spawn_obstacle_and_coin(&mut commands, Vec2::new(220.0, 0.0), ObstacleSize::Large);
}

fn play_collision_sound_system(mut collision_events: EventReader<CollisionEvent>) {
    for event in collision_events.iter() {
        match event {
            CollisionEvent(CollisionEventType::ObstacleCrash) => {
                println!("COLLISION with Obstacle!");
            }
            CollisionEvent(CollisionEventType::CoinCollect) => {
                println!("COLLISION with Coin!");
            }
        }
    }
}

fn score_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut game_score: ResMut<GameScore>,
) {
    for event in collision_events.iter() {
        match event {
            CollisionEvent(CollisionEventType::CoinCollect) => {
                game_score.score += 1;
            }
            CollisionEvent(CollisionEventType::ObstacleCrash) => {
                // Do nothing
            }
        }
    }
}

fn show_score_system(game_score: ResMut<GameScore>, mut query: Query<&mut Text, With<ScoreText>>) {
    for mut text in &mut query {
        text.sections[1].value = format!("{}", game_score.score);
    }
}
