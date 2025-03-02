use std::cmp::{max, min};
use std::net::{UdpSocket};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration};
use macroquad::prelude::*;
use flatbuffers::{root, FlatBufferBuilder, Push};
use flatbuffers;
use macroquad::miniquad::conf::Platform;
use macroquad::miniquad::window::{screen_size, set_window_size};

#[allow(dead_code, unused_imports)]
#[path = "../players_list_generated.rs"]
mod players_list_generated;
use crate::players_list_generated::{Color, PlayersList};
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};

const MAX_PLAYERS: usize = 10;
const GRAVITY: f32 = 1.0;
const FRICTION: f32 = 1.0;
const TICK_DURATION: Duration = Duration::from_millis(1);
const SERVER_ADDR: &str = "127.0.0.1:9000";
const CLIENT_ADDR: &str = "127.0.0.1:3000";
const SCALE: f32 = 2.0;
const PLAYER_SIZE: f32 = 16.0;

struct Player1 {
    id: Option<usize>,
    pos: Vec2,
    color: Color,
}

impl Player1 {
    fn new() -> Player1 {
        Player1 {
            id: None,
            pos: Vec2::ZERO,
            color: Color::Red,
        }
    }
}
struct OwnedPlayer {
    x: f32,
    y: f32,
    color: Color,
}

struct Resolution {
    width: f32,
    height: f32,
    scale: f32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Multi".to_owned(),
        window_width: 640,
        window_height: 360,
        high_dpi: false,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        icon: None,
        platform: Default::default(),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player1 {
        id: Some(0),
        pos: Vec2::ZERO,
        color: Color::Red,
    };

    let mut scale = 1.0;
    change_resolution(Resolution { width: 640.0, height: 360.0, scale: 1.0 }, &mut scale);

    let socket = Arc::new(UdpSocket::bind(CLIENT_ADDR).unwrap());
    let mut players: Arc<Mutex<Vec<OwnedPlayer>>> = Arc::new(Mutex::new(Vec::new()));
    let mut commands: Vec<PlayerCommand> = Vec::new();

    let tick_players: Arc<Mutex<Vec<OwnedPlayer>>> = Arc::clone(&players);
    let tick_socket = Arc::clone(&socket);

    thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            let (amt, src_addr) = socket.recv_from(&mut buf).unwrap();
            if src_addr.to_string() != SERVER_ADDR { continue; };
            let mut players_guard = players.lock().unwrap();
            handle_packet(&buf[..amt], &mut players_guard);

            drop(players_guard);
        }
    });

    loop {
        input_handler(&mut commands);
        if !commands.is_empty() {
            let mut builder = FlatBufferBuilder::with_capacity(2048);
            let commands_vec = builder.create_vector(&commands);
            let player_command = PlayerCommands::create(&mut builder, &PlayerCommandsArgs {
                commands: Some(commands_vec)
            });
            builder.finish(player_command, None);
            let bytes = builder.finished_data();
            tick_socket.send_to(&bytes, SERVER_ADDR).expect("Packet couldn't send.");
        }
        commands.clear();

        let players_guard = tick_players.lock().unwrap();
        render(&player, &players_guard, scale);
        drop(players_guard);
        next_frame().await;
    }
}

fn handle_packet(packet: &[u8], players: &mut Vec<OwnedPlayer>) {
    let players_list = root::<PlayersList>(packet).expect("No players received.");
    if let Some(player_vec) = players_list.players() {
        players.clear();
        for p in player_vec {
            players.push(OwnedPlayer { x: p.x(), y: p.y(), color: p.color() });
        }
    }
}

fn render(player: &Player1, players: &MutexGuard<Vec<OwnedPlayer>>, scale: f32) {
    clear_background(BLACK);
    for p in players.iter() {
        draw_rectangle(p.x * scale, p.y * scale, PLAYER_SIZE * scale, PLAYER_SIZE * scale, RED);
        //println!("{}, {}", p.x, p.y);
    }
}

fn input_handler(commands: &mut Vec<PlayerCommand>) {
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        commands.push(PlayerCommand::Move_right);
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        commands.push(PlayerCommand::Move_left);
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) {
        commands.push(PlayerCommand::Jump);
    }
}

fn change_resolution(resolution: Resolution, scale: &mut f32) {
    request_new_screen_size(resolution.width, resolution.height);
    *scale = resolution.scale;
}