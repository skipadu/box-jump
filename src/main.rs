use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

const PLAYER_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const PLAYER_COLOR: Color = Color::rgb(0.1, 0.8, 0.3);
const PLAYER_SPEED: f32 = 100.0;

const OBSTACLE_COLOR: Color = Color::rgb(0.8, 0.2, 0.1);
const OBSTACLE_SMALL_SIZE: Vec3 = Vec3::new(10.0, 10.0, 0.0);
const OBSTACLE_LARGE_SIZE: Vec3 = Vec3::new(30.0, 20.0, 0.0);

const COIN_SIZE: Vec3 = Vec3::new(5.0, 5.0, 0.0);
const COIN_COLOR: Color = Color::rgb(0.8, 0.7, 0.4);

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_systems(
            (check_collision, player_movement.before(check_collision))
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Coin;

#[derive(Default)]
struct CollisionEvent;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Draw player, a green box
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, PLAYER_SIZE.y / 2.0, 0.0),
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            ..default()
        },
        Player,
        // Collider,
    ));
    // Draw obstacle, a red box
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(50.0, OBSTACLE_SMALL_SIZE.y / 2.0, 0.0),
                scale: OBSTACLE_SMALL_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: OBSTACLE_COLOR,
                ..default()
            },
            ..default()
        },
        Obstacle,
        Collider,
    ));

    // "Coin" above obstacles to indicate when user has crossed
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(50.0, 20.0, 0.0),
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
        Coin,
        Collider,
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(220.0, OBSTACLE_LARGE_SIZE.y / 2.0, 0.0),
                scale: OBSTACLE_LARGE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: OBSTACLE_COLOR,
                ..default()
            },
            ..default()
        },
        Obstacle,
        Collider,
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(220.0, 30.0, 0.0),
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
        Coin,
        Collider,
    ));
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
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
}

fn check_collision(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    collider_query: Query<(Entity, &Transform, Option<&Obstacle>), With<Collider>>,
    coin_collider_query: Query<(Entity, &Transform, Option<&Coin>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (_entity, player_transform) = player_query.single_mut();
    let player_size = player_transform.scale.truncate();

    // Check collision with player vs obstacles
    for (_entity, transform, _maybe_obstacle) in &collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            // TODO: for play sound system
            // collision_events.send_default();
            println!("COLLISION with Obstacle!");
        }
    }
    // Check collision with player vs coins
    for (_entity, transform, _maybe_coin) in &coin_collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            println!("COLLISION with Coin!");
        }
    }
}
