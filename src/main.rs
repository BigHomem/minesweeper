#![allow(unused)]
use crossterm::event::{KeyEvent, KeyCode};
use rand::seq::index;
enum State {
    Normal(u8, bool),
    Clicked(u8, bool),
    Flagged(u8, bool),
}
impl State {
    fn render(&self, selected: bool, forced: bool) -> String {
        let icon = match self {
            Self::Clicked(x, false) => format!("{}", x),
            Self::Clicked(_, true) => "*".to_string(),
            Self::Flagged(_, _) => "⚑".to_string(),
            Self::Normal(_, true) if forced => "*".to_string(),
            &Self::Normal(x, false) if forced => format!("{}", x),
            _ => "□".to_string()
        };
        if selected { format!("{}◀", icon)} else {format!("{} ", icon)}
    }
    fn left_click(&mut self) -> bool {
        match self {
            Self::Normal(x, false) => *self = Self::Clicked(*x, false),
            Self::Normal(_, true) => {
                *self = Self::Clicked(99, true);
                return true;
            }
            _ => ()
        };
        false
    }
    fn right_click(&mut self) {
        match self {
            Self::Normal(x, y) => *self = Self::Flagged(*x, *y),
            Self::Flagged(x, y) => *self = Self::Normal(*x, *y),
            _ => ()
        }; 
    }
}
#[derive(Debug)]
enum Input {
    Quit,
    LeftClick,
    RightClick,
    Up,
    Down,
    Left,
    Right
}
fn get_input(x: i32, y: i32, l: i32) -> Input {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Char('z'), ..}) => break Input::LeftClick,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Char('x'), ..}) => break Input::RightClick,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Esc, ..}) => break Input::Quit,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Up, ..}) if !(x < y) => break Input::Up,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Down, ..}) if !(x - l > -y)=> break Input::Down,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Left, ..}) if !(x % y == 0) => break Input::Left,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Right, ..}) if !(x % y == y - 1) => break Input::Right,
            _ => ()
        }
    }
        
}
fn handle_input(game: &mut Vec<Tile>, index: &mut i32, width: i32, input: Input) -> bool {
    match input {
        Input::Up => *index = *index - width,
        Input::Down => *index = *index + width,
        Input::Left => *index = *index - 1,
        Input::Right => *index = *index + 1,
        Input::LeftClick => return game[*index as usize].state.left_click(),
        Input::RightClick => game[*index as usize].state.right_click(),
        Input::Quit => return true,
    };
    false
}
struct Tile {
    state: State,
}
fn get_neighbor_indexes(x: i32, y: i32, l: i32) -> Vec<i32> {
    let mut result = vec![];
    let (top, bot, left, right) = (x < y, x - l > -y, x % y == 0, x % y == y - 1);
    if !top { result.push(x - y); }
    if !bot { result.push(x + y); }
    if !left {
        result.push(x - 1);
        if !bot { result.push(x - 1 + y); }
        if !top { result.push(x - 1 - y); }
    }
    if !right {
        result.push(x + 1);
        if !bot { result.push(x + 1 + y); }
        if !top { result.push(x + 1 - y); }
    }
    result
}
fn gen_bomb_map(height: u32, width: u32, bomb_amount: usize) -> Vec<Option<u8>> {
    let map_length: u32 = width * height;
    let mut tiles: Vec<Option<u8>> = vec![Some(0); map_length as usize];

    let bombs = index::sample(&mut rand::thread_rng(), map_length as usize, bomb_amount).into_vec();
    for bomb in bombs {
        tiles[bomb] = None;
        for tile in get_neighbor_indexes(bomb as i32, width as i32, (map_length - 1) as i32) {
            tiles[tile as usize] = match tiles[tile as usize] {
                Some(x) => Some(x + 1),
                None => None
            } 
        }
    }
    
    tiles
}
fn new_game(height: u32, width: u32, bomb_amount: usize) -> Vec<Tile> {
    let mut game: Vec<Tile> = vec![];
    for tile in gen_bomb_map(height, width, bomb_amount) {
        game.push(match tile {
            Some(x) => Tile { state: State::Normal(x, false) },
            None => Tile { state: State::Normal(99, true) }
        })
    }
    game
}
fn render(game: &Vec<Tile>, width: u32, index: usize, forced: bool) {
    let render: Vec<String> = game.iter()
        .enumerate()
        .map(|(i, t)| t.state.render(i == index, forced))
        .collect();
    println!("\x1B[H\x1B[J");
    for row in render.chunks_exact(width as usize) {
        println!("{}", row.concat());
    }
}
fn main() {
    let (height, width, bomb_amount) = (15, 15, 50);
    let length = height * width;
    let mut game = new_game(height, width, bomb_amount);
    let mut selection: i32 = 14;
    loop {
        render(&game, width, selection as usize, false);
        let input = get_input(selection, width as i32, (length - 1) as i32);
        if handle_input(&mut game, &mut selection, width as i32, input) {break}
    }
    render(&game, width, selection as usize, true);
}