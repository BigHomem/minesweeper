use crossterm::event::{KeyEvent, KeyCode};
use rand::seq::index;

enum GameState {
    Won,
    Lost,
    Playing,
}
enum State {
    Normal(u8, bool),
    Clicked(u8, bool),
    Flagged(u8, bool),
}
impl State {
    fn render(&self, selected: bool, forced: bool) -> String {
        let icon = match self {
            Self::Clicked(0, false) => format!(" "),
            Self::Clicked(x, false) => format!("\x1B[38;5;240m{}", x),
            Self::Clicked(_, true) => "\x1B[38;5;160m*".to_string(),
            Self::Flagged(x, false) if forced => format!("\x1B[38;5;240m{}", x),
            Self::Flagged(_, _) => "\x1B[38;5;160m⚑".to_string(),
            Self::Normal(_, true) if forced => "\x1B[38;5;233m*".to_string(),
            Self::Normal(0, false) if forced => format!(" "),
            Self::Normal(x, false) if forced => format!("\x1B[38;5;240m{}", x),
            _ => "\x1B[38;5;250m▣".to_string()
        };
        if selected { format!("{}\x1B[38;5;46m◀", icon)} else {format!("{} ", icon)}
    }
    fn left_click(&mut self) -> GameState {
        match self {
            Self::Normal(x, false) => *self = Self::Clicked(*x, false),
            Self::Normal(_, true) => {
                *self = Self::Clicked(99, true);
                return GameState::Lost;
            }
            _ => ()
        };
        GameState::Playing
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
fn chain_dig(game: &mut Vec<Tile>, h: i32, w: i32) -> bool {
    let mut done = true;
    for i in 0..game.len() {

        if game[i].checked {continue}
        if let State::Normal(_, false) = game[i].state {} else {continue}
        
        for tile in get_neighbor_indexes(i as i32, w, h) {
            if let State::Clicked(0, false) = game[tile as usize].state {
                match game[i].state {
                    State::Normal(x, false) => {
                        done = false;
                        game[i].state = State::Clicked(x, false);
                    }
                    _ => ()
                }
            }
        }
    }
    done
}
fn get_input(x: i32, w: i32, l: i32) -> Input {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Char('z'), ..}) => break Input::LeftClick,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Char('x'), ..}) => break Input::RightClick,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Esc, ..}) => break Input::Quit,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Up, ..}) if !(x < w) => break Input::Up,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Down, ..}) if !(x - l > -w) => break Input::Down,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Left, ..}) if !(x % w == 0) => break Input::Left,
            crossterm::event::Event::Key(KeyEvent {code: KeyCode::Right, ..}) if !(x % w == w - 1) => break Input::Right,
            _ => ()
        }
    }
        
}
fn handle_input(game: &mut Vec<Tile>, index: &mut i32, width: i32, input: Input) -> GameState {
    match input {
        Input::Up => *index = *index - width,
        Input::Down => *index = *index + width,
        Input::Left => *index = *index - 1,
        Input::Right => *index = *index + 1,
        Input::LeftClick => return game[*index as usize].state.left_click(),
        Input::RightClick => game[*index as usize].state.right_click(),
        Input::Quit => return GameState::Lost,
    };
    GameState::Playing
}
struct Tile {
    state: State,
    checked: bool
}
fn check_win(game: &Vec<Tile>) -> GameState {
    for tile in game {
        if let State::Normal(_, false) = tile.state {return GameState::Playing}
        if let State::Flagged(_, false) = tile.state {return GameState::Playing}
        if let State::Normal(_, true) = tile.state {return GameState::Playing}
    }
    GameState::Won
}
fn get_neighbor_indexes(x: i32, w: i32, h: i32) -> Vec<i32> {
    let l = (h * w) - 1;
    let mut result = vec![];
    let (top, bot, left, right) = (x < w, x - l > -w, x % w == 0, x % w == w - 1);
    if !top { result.push(x - w); }
    if !bot { result.push(x + w); }
    if !left {
        result.push(x - 1);
        if !bot { result.push(x - 1 + w); }
        if !top { result.push(x - 1 - w); }
    }
    if !right {
        result.push(x + 1);
        if !bot { result.push(x + 1 + w); }
        if !top { result.push(x + 1 - w); }
    }
    result
}
fn gen_bomb_map(height: i32, width: i32, bomb_amount: usize) -> Vec<Option<u8>> {
    let map_length: i32 = width * height;
    let mut tiles: Vec<Option<u8>> = vec![Some(0); map_length as usize];

    let bombs = index::sample(&mut rand::thread_rng(), map_length as usize, bomb_amount).into_vec();
    for bomb in bombs {
        tiles[bomb] = None;
        for tile in get_neighbor_indexes(bomb as i32, width, height) {
            tiles[tile as usize] = match tiles[tile as usize] {
                Some(x) => Some(x + 1),
                None => None
            } 
        }
    }
    
    tiles
}
fn new_game(height: i32, width: i32, bomb_amount: usize) -> Vec<Tile> {
    let mut game: Vec<Tile> = vec![];
    for tile in gen_bomb_map(height, width, bomb_amount) {
        game.push(match tile {
            Some(x) => Tile { state: State::Normal(x, false), checked: false },
            None => Tile { state: State::Normal(99, true), checked: false }
        })
    }
    game
}
fn render(game: &Vec<Tile>, width: i32, index: usize, forced: bool) {
    let render: Vec<String> = game.iter()
        .enumerate()
        .map(|(i, t)| t.state.render(i == index, forced))
        .collect();
    println!("\x1B[H");
    for row in render.chunks_exact(width as usize) {
        println!("{}", row.concat());
    }
}
fn main() {
    let (height, width, bomb_amount) = (15, 15, 45);
    let length = height * width;
    let mut game = new_game(height, width, bomb_amount);
    let mut selection: i32 = 0;
    println!("\x1B[H\x1B[J");
    loop {
        render(&game, width, selection as usize, false);
        let input = get_input(selection, width, length - 1);
        if let GameState::Lost = handle_input(&mut game, &mut selection, width, input) {break}
        loop {
            if chain_dig(&mut game, height, width) {break}
        }
        if let GameState::Won = check_win(&game) {break}
        for tile in &mut game {
            tile.checked = false;
        }
    }
    render(&game, width, selection as usize, true);
}