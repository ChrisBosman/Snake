use std::collections::VecDeque;
use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::color;

use std::thread;
use std::sync::mpsc::channel;

// Parameters
const X:usize = 50;  // Max X
const Y:usize = 20;  // Max Y
const FRAME_TIME:u128 = 100;  //in ms

#[derive(PartialEq)]
#[derive(Debug)] // Remove this, was only for debugging
enum Direction {
    Right,
    Up,
    Left,
    Down
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq, PartialOrd)]
#[derive(Debug)] // Remove this, was only for debugging
enum BoardElement{
    Empty,
    Wall,
    Apple,
    SnakeH,
    SnakeV,
    SnakeDl,
    SnakeDr,
    SnakeUl,
    SnakeUr,
    SnakeHead,
    SnakeTail,

    NewEmpty,
    NewWall,
    NewApple,
    NewSnakeH,
    NewSnakeV,
    NewSnakeDl,
    NewSnakeDr,
    NewSnakeUl,
    NewSnakeUr,
    NewSnakeHead,
    NewSnakeTail,
}

fn main(){
    // Create a screen
    // let mut screen = Display::new(X,Y);
    // screen.clear;
    
    // Setup the input
    let stdin = stdin();
    // Setup stdout to use the rawmode
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Setup the thread for key input
    let (tprod,event_queue) = channel();
    thread::spawn(move || {
        // Read input and send into the channel
        for c in stdin.keys(){
            tprod.send(c).unwrap();
        }
    });

    // Setting up the game loop
    let mut is_running = true;
    let mut board: [BoardElement;X*Y] = [BoardElement::Empty;Y*X];
    let mut apple: usize = usize::MAX;
    let mut snake: VecDeque<usize> = VecDeque::new();
    snake.push_back(X/2 + X*(Y/2));
    board[snake[0]] = BoardElement::NewSnakeHead;
    write!(stdout, r#"{}{}{}"#, termion::cursor::Goto(1, 1), termion::clear::All, termion::cursor::Hide).unwrap(); // Clear screen
    draw_board(&mut board);
    print_board(&board);

    apple = 42+16*X;
    
    // Game loop
    while is_running{
        let start_time = std::time::Instant::now(); 
        // Procces input
        for c in event_queue.try_iter(){
            match c.unwrap(){
                Key::Char('i') => move_apple(&mut board, &mut apple, Direction::Up),
                Key::Char('k') => move_apple(&mut board, &mut apple, Direction::Down),
                Key::Char('j') => move_apple(&mut board, &mut apple, Direction::Left),
                Key::Char('l') => move_apple(&mut board, &mut apple, Direction::Right),

                Key::Char('w') => is_running = move_snake(&mut board, &mut snake, Direction::Up),
                Key::Char('s') => is_running = move_snake(&mut board, &mut snake, Direction::Down),
                Key::Char('a') => is_running = move_snake(&mut board, &mut snake, Direction::Left),
                Key::Char('f') => is_running = move_snake(&mut board, &mut snake, Direction::Right),
                
                Key::Ctrl('q') => is_running = false,
                Key::Alt('q') => is_running = false,
                _ => (),
            }
        }
        // Update

        // Render
        write!(stdout, r#"{}"#, termion::cursor::Goto(1, 1)).unwrap(); // Clear screen        
        update_board(&mut board);
        stdout.flush().unwrap();

        // Take care of the frame rate
        let elapsed_time = start_time.elapsed().as_millis();
        if elapsed_time < FRAME_TIME{
            // Sleep until it is time for the new frame to start
            let sleep_time = FRAME_TIME-elapsed_time;
            thread::sleep(std::time::Duration::from_millis(sleep_time.try_into().unwrap()));
        }   
    }
    write!(stdout, r#"{}{}{}"#, termion::cursor::Goto(1, 1), termion::clear::All, termion::cursor::Show).unwrap(); // Clear screen
}

// Render the whole board
fn print_board(board : &[BoardElement;X*Y]){
    for index in 0..X*Y {
        match board[index]{
            BoardElement::Wall =>{
                // Print the walls (takes some time)
                // Check corners
                let _bl = X*(Y-1)+1;  // Bottom Left
                let _br = (X*Y)-1;  // Bottom right
                if index == 0               {print!("{} ╔",color::Fg(color::White))}
                else if index == X-1        {print!("{}═╗",color::Fg(color::White))}
                else if index == (Y-1)*X  {print!("{} ╚",color::Fg(color::White))}
                else if index == X*Y-1      {print!("{}═╝",color::Fg(color::White))}
                // Check sides
                else if index < X            {print!("{}══",color::Fg(color::White))}
                else if index > X*(Y-1) {print!("{}══",color::Fg(color::White))}
                else                    {print!("{} ║",color::Fg(color::White))}
            },
            BoardElement::Apple => print!(" {}■",color::Fg(color::Red)),
            BoardElement::SnakeH => print!(" {}█",color::Fg(color::Green)),
            BoardElement::SnakeHead => print!("{}██",color::Fg(color::Green)),
            _ => print!(" 0"),
        }
        if index % X == X-1 {
           print!("\r\n");
        }
    }
}

// Populate the board (with walls)
fn draw_board(board : &mut [BoardElement;X*Y]){
    for x in 0..X {
        for y in 0..Y {
            let index = x+y*X; 
            if x==0 { board[index] = BoardElement::Wall; continue;}
            if x==X-1 { board[index] = BoardElement::Wall; continue;}
            if y==0 { board[index] = BoardElement::Wall; continue;}
            if y==Y-1 { board[index] = BoardElement::Wall; continue;}
            board[index] = BoardElement::Empty;
        }
    }
}

// Render the new updates (and update the board)
fn update_board(board : &mut [BoardElement;X*Y]){
    for index in 0..X*Y {
        if board[index] < BoardElement::NewEmpty {continue;}

        print!(r#"{}"#,termion::cursor::Goto((index % X * 2 + 1).try_into().unwrap(),(index / X + 1).try_into().unwrap()));
        match board[index]{
            BoardElement::NewWall =>{
                // Print the walls (takes some time)
                // Check corners
                let _bl = X*(Y-1)+1;  // Bottom Left
                let _br = (X*Y)-1;  // Bottom right
                if index == 0               {print!("{} ╔",color::Fg(color::White))}
                else if index == X-1        {print!("{}═╗",color::Fg(color::White))}
                else if index == (Y-1)*X  {print!("{} ╚",color::Fg(color::White))}
                else if index == X*Y-1      {print!("{}═╝",color::Fg(color::White))}
                // Check sides
                else if index < X            {print!("{}══",color::Fg(color::White))}
                else if index > X*(Y-1) {print!("{}══",color::Fg(color::White))}
                else                    {print!("{} ║",color::Fg(color::White))}
                board[index] = BoardElement::Empty;
            },
            BoardElement::NewApple => {print!(" {}■",color::Fg(color::Red)); 
                                        board[index] = BoardElement::Apple},
            BoardElement::NewSnakeH => {print!(" {}█",color::Fg(color::Green));
                                        board[index] = BoardElement::SnakeH;},
            BoardElement::NewSnakeHead => {print!("{}██",color::Fg(color::Green));
                                        board[index] = BoardElement::SnakeHead;},
            _ => print!("{}  ",color::Fg(color::White)),
        }
    }
}

fn move_apple(board : &mut [BoardElement;X*Y], apple : &mut usize, dir: Direction){
    // Check if an apple exists
    if *apple == usize::MAX { return; } 
    // Remove previous apple
    board[*apple] = BoardElement::NewEmpty;

    // Update location
    match dir{
        Direction::Right => if *apple % X < X-2 { *apple += 1; },
        Direction::Left => if *apple % X > 1 { *apple -= 1; },
        Direction::Down => if *apple <= (X*(Y-2)) { *apple += X; },
        Direction::Up => if *apple >= 2*X { *apple -= X; },
    }
    // Place new apple
    board[*apple] = BoardElement::NewApple;
}

fn move_snake(board : &mut [BoardElement;X*Y], snake : &mut VecDeque<usize>, dir: Direction) -> bool {
    if snake.is_empty() { print!("Length too small, length = {}\n",snake.len()); return false; }
    // Get next spot
    let mut next_index: usize = 0;
    match dir{
        Direction::Right => { if let Some(last) = snake.back() {next_index = *last + 1}},
        Direction::Left => { if let Some(last) = snake.back() {next_index = *last - 1}},
        Direction::Down => { if let Some(last) = snake.back() {next_index = *last + X}},
        Direction::Up =>  { if let Some(last) = snake.back() {next_index = *last - X}},
    }
    // Check collision
    if next_index >= X*Y { print!("Index out of bounds, i = {}\n",next_index); return false; }
    if board[next_index] != BoardElement::Empty && board[next_index] != BoardElement::Apple { print!("That is a wall, i = {}\nElement = {:?}\n",next_index, board[next_index]); return false; }

    // Update the body
    // First delete the tail if it does not eat an appel
    if board[next_index] != BoardElement::Apple { 
        if let Some(_first) = snake.pop_front(){ board[_first] = BoardElement::NewEmpty }
    }
    
    // Update all the element in between
    // W.N.I.P.

    // Place the first element
    board[next_index] = BoardElement::NewSnakeHead;
    snake.push_back(next_index);

    return true;
}

/*
    Meaning of board
        0 = Nothing
        1 = Wall
        2 = Object ¯\_(ツ)_/¯
        3 = ?
        4 = ?
        5 = ?
        6 = ?
        7 = Apple
        8 = Snake Body
        9 = Snake Head

    For updating
        An offset (CHANGE_OFFSET) will be added to the value to let it know that it is new.
        And when it will be drawn it will deduct the offset from it to signal that it is not new anymore
*/