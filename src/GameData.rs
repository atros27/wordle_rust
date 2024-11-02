//use std::intrinsics::size_of;
use crate::ScreenState::ScreenElement;
use femtovg::{renderer::Renderer, Align, Baseline, Canvas, Color, Paint, Path};
pub use glutin::{event_loop, window::WindowBuilder, ContextBuilder, ContextWrapper};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::fs;
use std::process;
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    pub static ref GREY: Color = Color::hex("999999");
    pub static ref DARK_GREY: Color = Color::hex("666666");
    pub static ref YELLOW: Color = Color::hex("FFFF00");
    pub static ref GREEN: Color = Color::hex("00FF00");
    pub static ref BLACK: Color = Color::hex("000000");
    pub static ref WHITE: Color = Color::hex("FFFFFF");
    pub static ref DEFAULT_LETTER_BLOCK: LetterBlock = LetterBlock {
        letter: 'c',
        fill_color: Paint::color(*GREY),
        width: 50.0,
        height: 50.0,
        x: 0.0,
        y: 0.0,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LetterBlock {
    pub(crate) letter: char,
    fill_color: Paint,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl<R: Renderer> ScreenElement<R> for LetterBlock {
    //type R = OpenGl;
    fn render(&self, canvas: &mut Canvas<R>) {
        let mut path = Path::new();
        path.rounded_rect(self.x, self.y, self.width, self.height, 5.0);
        canvas.fill_path(&mut path, self.fill_color);
        canvas.stroke_path(&mut path, self.fill_color);

        let middle_x = self.x + self.width / 2.0;
        let middle_y = self.y + self.height / 2.0;
        let mut text_paint = Paint::color(*WHITE);
        //text_paint.set_font();
        text_paint.set_font_size(36.0);
        text_paint.set_text_align(Align::Center);
        text_paint.set_text_baseline(Baseline::Middle);
        let _ = canvas
            .fill_text(middle_x, middle_y, self.letter.to_string(), text_paint)
            .expect("Text Render ERROR");
    }
}

// pub struct LetterBlockRow<'a> {
//     as_string: &'a str,
//     start_x: f32,
//     start_y: f32,
//     spacing: f32,
//     pub(crate) letters: Vec<LetterBlock>,
// }

pub struct Cursor {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

pub struct GameData {
    pub(crate) secret_word: String,
    pub(crate) entered_letters: Vec<Vec<char>>,
    pub(crate) word_attempts: Vec<String>,
    pub(crate) attempt_letter_blocks: Vec<Vec<LetterBlock>>,
    pub(crate) keyboard_state: HashMap<char, KeyState>,
    pub(crate) keyboard_letter_blocks: Vec<LetterBlock>,
    pub(crate) cursor: Cursor,
}

pub fn one_below(prev: &LetterBlock) -> f32 {
    let space = 5.0;
    prev.y + prev.height + space
}

pub fn init_row(as_string: &str, start_x: f32, start_y: f32, spacing: f32) -> Vec<LetterBlock> {
    let mut letter_row = Vec::new();
    for i in 0..as_string.len() {
        letter_row.push(LetterBlock {
            letter: as_string.chars().nth(i).unwrap(),
            x: start_x + (i as f32) * DEFAULT_LETTER_BLOCK.width + ((i as f32) - 1.0) * spacing,
            y: start_y,
            ..*DEFAULT_LETTER_BLOCK
        });
    }
    letter_row
}

impl<'a> GameData {
    pub(crate) fn new() -> GameData {
        let mut game_data = GameData {
            secret_word: "".to_string(),
            entered_letters: vec![],
            word_attempts: vec![],
            attempt_letter_blocks: vec![],
            keyboard_state: HashMap::new(),
            keyboard_letter_blocks: vec![],
            cursor: Cursor { row: 0, col: 0 },
        };
        let LEFT_MARGIN = 0.0;
        let SPACING = 5.0;
        game_data
            .attempt_letter_blocks
            .push(init_row("     ", LEFT_MARGIN, 0.0, SPACING));
        let mut start_y = 0.0;
        for i in 1..5 {
            start_y = one_below(&game_data.attempt_letter_blocks.last().unwrap()[0]);
            game_data
                .attempt_letter_blocks
                .push(init_row("     ", LEFT_MARGIN, start_y, SPACING));
        }
        start_y = 100.0 + one_below(&game_data.attempt_letter_blocks.last().unwrap()[0]);
        game_data.keyboard_letter_blocks.append(&mut init_row(
            "QWERTYUIOP",
            LEFT_MARGIN,
            start_y,
            SPACING,
        ));
        start_y = one_below(&game_data.keyboard_letter_blocks.last().unwrap());
        game_data.keyboard_letter_blocks.append(&mut init_row(
            "ASDFGHJKL",
            LEFT_MARGIN,
            start_y,
            SPACING,
        ));
        start_y = one_below(&game_data.keyboard_letter_blocks.last().unwrap());
        game_data.keyboard_letter_blocks.append(&mut init_row(
            "ZXCVBNM",
            LEFT_MARGIN,
            start_y,
            SPACING,
        ));

        let first_letter = 'A';
        for i in 0..26 {
            let current_letter = std::char::from_u32(first_letter as u32 + i).unwrap();
            game_data
                .keyboard_state
                .insert(current_letter, KeyState::Unused);
        }
        let word_string = fs::read_to_string("sgb-words-trimmed.txt").unwrap();
        let words: Vec<&str> = word_string.split("\n").collect();
        game_data.secret_word = words
            .choose(&mut thread_rng())
            .expect("word list empty or malformed")
            .to_string();
        game_data
    }
    pub(crate) fn can_type(&self) -> bool {
        //Cursor stays on last block of row until validation, then reset to next row, first block
        //If cursor already has text, can't type
        let row: usize = self.cursor.row;
        let col: usize = self.cursor.col;
        self.entered_letters[row][col] != ' '
    }
    pub(crate) fn type_char(&mut self, c: &char) {
        self.entered_letters[self.cursor.row][self.cursor.col] = *c;
        self.blink(c);
        if self.cursor.col < 4 {
            self.cursor.col += 1;
        }
    }
    pub(crate) fn remove_char(&mut self) {
        self.entered_letters[self.cursor.row][self.cursor.col] = ' ';
        self.cursor.col -= 1;
        self.entered_letters[self.cursor.row][self.cursor.col] = ' ';
    }
    pub(crate) fn blink(&mut self, c: &char) {
        let mut key_index = self
            .keyboard_letter_blocks
            .iter()
            .enumerate()
            .find(|(_, key)| key.letter == *c)
            .unwrap()
            .0;
        if let Some(state) = self.keyboard_state.get(c) {
            //If pressed key is in keyboard
            if *state == KeyState::Unused {
                //If key is not colored
                self.keyboard_letter_blocks[key_index]
                    .fill_color
                    .set_color(*DARK_GREY);
                sleep(Duration::from_millis(50));
                self.keyboard_letter_blocks[key_index]
                    .fill_color
                    .set_color(*GREY);
            }
        }
    }
    pub(crate) fn verify(&mut self) {
        let word: String = self.entered_letters[self.cursor.row].iter().collect();
        //self.word_attempts.push(&word);
        let mut num_correct = 0;
        for n in 0..5 {
            if word.chars().nth(n) == self.secret_word.chars().nth(n) {
                //Exact Match
                self.attempt_letter_blocks[self.cursor.row][n].fill_color = Paint::color(*GREEN);
                self.keyboard_state
                    .insert(self.entered_letters[self.cursor.row][n], KeyState::Green);
                num_correct += 1;
            } else if self
                .secret_word
                .chars()
                .collect::<Vec<char>>()
                .contains(&word.chars().nth(n).unwrap())
            {
                //Out-of-order Match
                self.attempt_letter_blocks[self.cursor.row][n].fill_color = Paint::color(*YELLOW);
                self.keyboard_state
                    .insert(self.entered_letters[self.cursor.row][n], KeyState::Yellow);
            } else {
                //Incorrect
                self.attempt_letter_blocks[self.cursor.row][n].fill_color =
                    Paint::color(*DARK_GREY);
                self.keyboard_state
                    .insert(self.entered_letters[self.cursor.row][n], KeyState::Used);
            }
            sleep(Duration::from_millis(200));
        }
        if num_correct == 5 {
            //Win state
            print!("You won! Good girl! *Headpats*");
            sleep(Duration::from_secs(5));
            process::exit(0);
        } else if self.cursor.row < 4 {
            //Next try
            self.cursor.row += 1;
            self.cursor.col = 0;
        } else {
            //Out of Tries
            print!("You lose! Bad Girl!! >:(");
            let word_reveal = format!("The word was: {}", self.secret_word);
            print!("{}", word_reveal);
            sleep(Duration::from_secs(5));
            process::exit(0);
        }
    }
}
#[derive(PartialEq)]
pub enum KeyState {
    Green,
    Yellow,
    Used,
    Unused,
}
