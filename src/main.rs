use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};
use macroquad::prelude::*;
use flatbuffers::{root, FlatBufferBuilder, Push};
use flatbuffers;
use macroquad::miniquad::window::set_window_size;

#[allow(dead_code, unused_imports)]
#[path = "../players_list_generated.rs"]
mod players_list_generated;
use crate::players_list_generated::{Color, Player, PlayersList, PlayersListArgs};
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};

const MAX_PLAYERS: usize = 10;
const GRAVITY: f32 = 1.0;
const FRICTION: f32 = 1.0;

const TICK_DURATION: Duration = Duration::from_millis(1);

struct Player1 {
    id: Option<usize>,
    pos: Vec2,
    vel: Vec2,
    acc: f32,
    jump_force: f32,
    color: Color,
}

impl Player1 {
    fn new() -> Player1 {
        Player1 {
            id: None,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            acc: 1.0,
            jump_force: 10.0,
            color: Color::Red,
        }
    }
}
struct OwnedPlayer {
    x: f32,
    y: f32,
    color: Color
}

#[macroquad::main("Multi")]
async fn main() {
    let mut player = Player1 {
        id: Some(0),
        pos: Vec2::ZERO,
        vel: Vec2::ZERO,
        acc: 1.0,
        jump_force: 10.0,
        color: Color::Red,
    };

    set_window_size(600,400);
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:3003").unwrap());
    let mut players: Arc<Mutex<Vec<OwnedPlayer>>> = Arc::new(Mutex::new(Vec::new()));
    let mut commands: Vec<PlayerCommand> = Vec::new();

    let tick_players: Arc<Mutex<Vec<OwnedPlayer>>> = Arc::clone(&players);
    let tick_socket = Arc::clone(&socket);

    thread::spawn(move || {
        let mut buf = [0u8; 2048];
       loop {
           let (amt, src_addr) = socket.recv_from(&mut buf).unwrap();
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
            let bytes= builder.finished_data();
            tick_socket.send_to(&bytes, "127.0.0.1:9000").expect("Packet couldn't send.");
        }
        commands.clear();

        physics(&mut player);
        let players_guard = tick_players.lock().unwrap();
        render(&player, &players_guard);
        drop(players_guard);
        next_frame().await;
    }
}

fn handle_packet(packet: &[u8], players: &mut Vec<OwnedPlayer>) {
    //println!("Packet received from server");
    let players_list = root::<PlayersList>(packet).expect("No players received.");
    if let Some(player_vec) = players_list.players() {
        println!("{:?}", player_vec);
        players.clear();
        for p in player_vec {
            players.push(OwnedPlayer { x: p.x(), y: p.y(), color: p.color()});
        }
    }
}

fn physics(player: &mut Player1) {
    /*player.pos.x = player.pos.x + player.vel.x;
    player.pos.y = player.pos.y + player.vel.y;
    player.vel.x *= FRICTION;
    player.vel.y += GRAVITY;

    if player.pos.y > screen_height() - 10.0 {
        player.pos.y = screen_height() - 10.0;
        player.vel.y = 0.0;
    }
    if player.pos.y < 0.0 {
        player.pos.y = 0.0;
        player.vel.y = 0.0;
    }
    if player.pos.x > screen_width() - 10.0 {
        player.pos.x = screen_width() - 10.0;
        player.vel.x = 0.0;
    }
    if player.pos.x < 0.0 {
        player.pos.x = 0.0;
        player.vel.x = 0.0;
    }*/
}

fn render(player: &Player1, players: &MutexGuard<Vec<OwnedPlayer>>) {
    clear_background(BLACK);
    for p in players.iter() {
        draw_rectangle(p.x, p.y, 10.0, 10.0, RED);
        println!("{}, {}", p.x, p.y);
    }
}

fn input_handler(commands: &mut Vec<PlayerCommand>) {
    if is_key_down(KeyCode::Right) {
        println!("Moving right");
        commands.push(PlayerCommand::Move_right);
    }
    if is_key_down(KeyCode::Left) {
        commands.push(PlayerCommand::Move_left);
    }
    if is_key_pressed(KeyCode::Up) {
        commands.push(PlayerCommand::Jump);
    }
}
