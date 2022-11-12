use std::collections::VecDeque;
use std::io::{stdin, stdout, Write};
use rand::rngs::ThreadRng;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::color;

use std::thread;
use std::sync::mpsc::channel;
use rand::Rng;
// Parameters
const X:u16 = 50;  // Max X
const Y:u16 = 20;  // Max Y
const FRAME_TIME:u128 = 100;  //in ms

#[derive(Clone)]
#[derive(Copy)]
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
    Snake,

    NewEmpty,
    // NewWall,
    NewApple,
    NewSnake,
}

fn main(){
    game();
}

fn game(){
    // -------- {Set up everything} --------- \\
    let mut err_msg:String = "".into();
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
            tprod.send(c).expect("KeyPress");
        }
    });
    let mut is_running = true;
    let mut rng = rand::thread_rng();
    // ------------ {Start of the program} ------------ \\
    while is_running {
        // ------------ {The Start Screen} ------------ \\
        // No
        // ------------ {The Snake game} ------------ \\
        // Setting up the game loop
        let mut is_playing = true;
        let mut board: [BoardElement;(X*Y) as usize] = [BoardElement::Empty;(X*Y) as usize];
        let mut apple: u16 = u16::MAX;
        let mut direction: Direction = Direction::Right;
        let mut old_dir: Direction = Direction::Left;
        let mut snake: VecDeque<u16> = VecDeque::new();
        snake.push_back(X/2 + X*(Y/2));
        snake.push_back(X/2 - 1 + X*(Y/2));
        board[snake[0] as usize] = BoardElement::NewSnake;
        board[snake[1] as usize] = BoardElement::NewSnake;

        write!(stdout, r#"{}{}{}"#, termion::cursor::Goto(1, 1), termion::clear::All, termion::cursor::Hide).unwrap(); // Clear screen
        draw_board(&mut board);
        print_board(&board);

        replace_apple(&mut board, &mut apple, &mut rng);
        
        // Game loop
        'playloop: while is_playing{
            let start_time = std::time::Instant::now(); 
            // Procces input
            for c in event_queue.try_iter(){
                match c.unwrap(){
                    Key::Char('i') => move_apple(&mut board, &mut apple, Direction::Up),
                    Key::Char('k') => move_apple(&mut board, &mut apple, Direction::Down),
                    Key::Char('j') => move_apple(&mut board, &mut apple, Direction::Left),
                    Key::Char('l') => move_apple(&mut board, &mut apple, Direction::Right),

                    Key::Char('w') => if old_dir != Direction::Down  {direction = Direction::Up},
                    Key::Char('s') => if old_dir != Direction::Up  {direction = Direction::Down},
                    Key::Char('a') => if old_dir != Direction::Right  {direction = Direction::Left},
                    Key::Char('d') => if old_dir != Direction::Left  {direction = Direction::Right},
                    
                    Key::Ctrl('q') => break 'playloop,
                    Key::Alt('q') => break 'playloop,
                    _ => (),
                }
            }
            // Update
            is_playing = move_snake(&mut board, &mut snake, &direction, &mut err_msg, &mut apple, &mut rng);
            old_dir = direction;

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

        write!(stdout, r#"{}"#, termion::cursor::Goto((X/2).try_into().unwrap(), (Y/2).try_into().unwrap())).unwrap(); // Clear screen 
        print!("GAME OVER\n Press R to restart\n Press q to quit\n"); 
        stdout.flush().unwrap();
        // ------------ {Death Screen} ------------ \\
        'gameOver:  loop {
            // Procces input
            for c in event_queue.try_iter(){
                match c.unwrap(){
                    Key::Char('r') => break 'gameOver,
                    Key::Char('q') => {is_running = false; break 'gameOver},
                    Key::Alt('q') => {is_running = false; break 'gameOver},
                    _ => (),
                }
            }  
        }
    }
    write!(stdout, r#"{}{}{}"#, termion::cursor::Goto(1, 1), termion::clear::All, termion::cursor::Show).unwrap(); // Clear screen
}

// Render the whole board
fn print_board(board : &[BoardElement;(X*Y) as usize]){
    for index in 0..X*Y {
        match board[index as usize]{
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
            BoardElement::Snake => print!("{}██",color::Fg(color::Green)),
            _ => print!("  "),
        }
        if index % X == X-1 {
           print!("\r\n");
        }
    }
}

// Populate the board (with walls)
fn draw_board(board : &mut [BoardElement;(X*Y) as usize]){
    for x in 0..X {
        for y in 0..Y {
            let index = x+y*X; 
            if x==0 { board[index as usize] = BoardElement::Wall; continue;}
            if x==X-1 { board[index as usize] = BoardElement::Wall; continue;}
            // if y==0 { board[index as usize] = BoardElement::Wall; continue;}
            // if y==Y-1 { board[index as usize] = BoardElement::Wall; continue;}
            board[index as usize] = BoardElement::Empty;
        }
    }
}

// Render the new updates (and update the board)
fn update_board(board : &mut [BoardElement;(X*Y) as usize]){
    for index in 0..X*Y {
        if board[index as usize] < BoardElement::NewEmpty {continue;}

        print!(r#"{}"#,termion::cursor::Goto((index % X * 2 + 1).try_into().unwrap(),(index / X + 1).try_into().unwrap()));
        match board[index as usize]{
            // BoardElement::NewWall =>{
            //     // Print the walls (takes some time)
            //     // Check corners
            //     let _bl = X*(Y-1)+1;  // Bottom Left
            //     let _br = (X*Y)-1;  // Bottom right
            //     if index == 0               {print!("{} ╔",color::Fg(color::White))}
            //     else if index == X-1        {print!("{}═╗",color::Fg(color::White))}
            //     else if index == (Y-1)*X  {print!("{} ╚",color::Fg(color::White))}
            //     else if index == X*Y-1      {print!("{}═╝",color::Fg(color::White))}
            //     // Check sides
            //     else if index < X            {print!("{}══",color::Fg(color::White))}
            //     else if index > X*(Y-1) {print!("{}══",color::Fg(color::White))}
            //     else                    {print!("{} ║",color::Fg(color::White))}
            //     board[index as usize] = BoardElement::Empty;
            // },
            BoardElement::NewApple => {print!(" {}■",color::Fg(color::Red)); 
                                        board[index as usize] = BoardElement::Apple},
            BoardElement::NewSnake => {print!("{}██",color::Fg(color::Green));
                                        board[index as usize] = BoardElement::Snake;},
            _ => print!("{}  ",color::Fg(color::White)),
        }
    }
}

fn move_apple(board : &mut [BoardElement;(X*Y) as usize], apple : &mut u16, dir: Direction){
    // Check if an apple exists
    if *apple == u16::MAX { return; } 
    // Remove previous apple
    board[*apple as usize] = BoardElement::NewEmpty;

    // Update location
    match dir{
        Direction::Right => if *apple % X < X-2 { *apple += 1; },
        Direction::Left => if *apple % X > 1 { *apple -= 1; },
        Direction::Down => if *apple <= (X*(Y-2)) { *apple += X; },
        Direction::Up => if *apple >= 2*X { *apple -= X; },
    }
    // Place new apple
    board[*apple as usize] = BoardElement::NewApple;
}

fn move_snake(board : &mut [BoardElement;(X*Y) as usize], snake : &mut VecDeque<u16>, dir: &Direction, err_msg : &mut String, apple : &mut u16 ,  rng: &mut ThreadRng) -> bool {
    if snake.is_empty() {  *err_msg = "Snake length too small".to_string(); return false; }
    // Get next spot
    let next_index: u16;
    let snake_x: u16; let snake_y:u16; let head_index: u16;
    if let Some(last) = snake.back() {
        snake_x = *last % X;
        snake_y = *last/X;
        head_index = *last;
    }else{
        *err_msg = "Error while reading snake head position".to_string();
        return false;
    }

    // Get new position (if at an edge without a wall then portal though and arive at the other side)
    match dir{
        Direction::Right => { if snake_x == X-1 {next_index = 0     + snake_y*X} else {next_index = head_index + 1;}},
        Direction::Left => {  if snake_x == 0   {next_index = X-1   + snake_y*X} else {next_index = head_index - 1;}},
        Direction::Down => {  if snake_y == Y-1 {next_index = snake_x +   0    } else {next_index = head_index + X;}},
        Direction::Up  =>  {  if snake_y == 0   {next_index = snake_x + (Y-1)*X} else {next_index = head_index - X;}},
    }
    // Check collision
    if next_index >= X*Y { *err_msg = "Index out of bounds".to_string(); return false; }

    if board[next_index as usize] != BoardElement::Empty && board[next_index as usize] != BoardElement::Apple && board[next_index as usize] != BoardElement::NewEmpty { *err_msg = "That is a wall".to_string(); return false; }

    // Update the body
    // First delete the tail if it does not eat an appel
    if board[next_index as usize] != BoardElement::Apple { 
        if let Some(_first) = snake.pop_front(){ board[_first as usize] = BoardElement::NewEmpty }
    }
    else{
        replace_apple(board, apple, rng)
    }

    // Place the first element
    board[next_index as usize] = BoardElement::NewSnake;
    snake.push_back(next_index);

    return true;
}

fn replace_apple(board : &mut [BoardElement;(X*Y) as usize], apple : &mut u16, rng: &mut ThreadRng){
    // Remove previous apple
    if *apple != u16::MAX { 
        board[*apple as usize] = BoardElement::NewEmpty;
    }

    let place_x = rng.gen_range(1..(X-1));
    let place_y = rng.gen_range(1..(Y-1));
    *apple = place_x + place_y*X;

    // Place new apple
    board[*apple as usize] = BoardElement::NewApple;
}

/*
    For updating
        An offset (CHANGE_OFFSET) will be added to the value to let it know that it is new.
        And when it will be drawn it will deduct the offset from it to signal that it is not new anymore
*/