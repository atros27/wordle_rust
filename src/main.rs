mod GameData;
mod ScreenState;

use crate::event_loop::{ControlFlow, EventLoop};
use crate::GameData::{LetterBlock, DARK_GREY, GREEN, YELLOW};
use crate::ScreenState::ScreenData;
use event::EventPublisher;
use femtovg::{renderer::OpenGl, renderer::Renderer, Align, Baseline, Canvas, Color, Paint, Path};
use glutin::event::{Event, VirtualKeyCode, WindowEvent};
use glutin::window::Window;
use glutin::PossiblyCurrent;
pub use glutin::{event_loop, window::WindowBuilder, ContextBuilder, ContextWrapper};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::io::Result;
use std::process;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

#[macro_use]
extern crate lazy_static;

pub fn one_over(prev: &LetterBlock) -> f32 {
    let space = 5.0;
    &prev.x + &prev.width + space
}
pub fn one_below(prev: &LetterBlock) -> f32 {
    let space = 5.0;
    &prev.y + &prev.height + space
}

fn main() {
    //Instantiate OpenGl Context
    let window_size = glutin::dpi::PhysicalSize::new(1000, 1000);
    let event_loop = event_loop::EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(window_size.clone())
        .with_title("Gradient Test");
    let window_context = ContextBuilder::new()
        .build_windowed(wb, &event_loop)
        .unwrap();
    let mut window_context = unsafe { window_context.make_current().unwrap() };

    //Create Rendering and Canvas Objects
    let render_obj = OpenGl::new_from_glutin_context(&window_context)
        .ok()
        .unwrap();
    let mut canvas = Canvas::new(render_obj).expect("Cannot create canvas");
    canvas
        .add_font("SpaceGrotesk-Bold.ttf")
        .expect("TODO: panic message");
    canvas.set_size(
        window_size.width as u32,
        window_size.height as u32,
        window_context.window().scale_factor() as f32,
    );

    run_game(event_loop, window_context, canvas);
}

fn run_game<R: Renderer + 'static>(
    event_loop: EventLoop<()>,
    mut window_context: ContextWrapper<PossiblyCurrent, Window>,
    canvas: Canvas<R>,
) {
    let mut game_data: GameData::GameData = GameData::GameData::new();
    let mut screen_data: ScreenData<'static, R> = ScreenData::init(canvas);

    event_loop.run(move |event: Event<'_, ()>, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        event_logic::<R>(event, &mut game_data, &mut screen_data, &mut window_context);
    });
}

fn event_logic<R: Renderer>(
    event: Event<()>,
    game_data: &mut GameData::GameData,
    screen_data: &mut ScreenData<R>,
    window_context: &mut ContextWrapper<PossiblyCurrent, Window>,
) {
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::ReceivedCharacter(c) => {
                if game_data.can_type() {
                    //A-Z letter input with space to go
                    game_data.type_char(&c);
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                if let Some(virtual_key_code) = input.virtual_keycode {
                    if (virtual_key_code == VirtualKeyCode::Return
                        || virtual_key_code == VirtualKeyCode::NumpadEnter)
                        && (game_data.cursor.col == 4)
                        && !game_data.can_type()
                    {
                        //Full word and ENTER selected
                        //Evaluate word
                        game_data.verify();
                    } else if virtual_key_code == VirtualKeyCode::Back && game_data.cursor.col > 0 {
                        //Delete letter
                        game_data.remove_char();
                    } else if virtual_key_code == VirtualKeyCode::Escape {
                        process::exit(0);
                    }
                }
            }
            _ => (),
        },
        Event::RedrawRequested(_) => {
            screen_data.init_frame(&game_data);
            for element in screen_data.screen_elements.iter() {
                element.render(&mut screen_data.canvas);
            }
            let now = Instant::now();
            let dt = (now - screen_data.prevt).as_secs_f32();
            screen_data.prevt = now;
            screen_data.perf.update(dt);

            screen_data.canvas.save();
            screen_data.canvas.reset();
            //perf.render(&mut canvas, 5.0, 5.0);
            screen_data.canvas.restore();
            screen_data.canvas.flush();
            window_context.swap_buffers().unwrap();
        }
        _ => (),
    }
}

// fn trim_wordlist() -> Result<()> {
//     let words = fs::read_to_string("sgb-words.txt")
//         .unwrap()
//         .split('\n')
//         .filter(|&word| word.chars().into_iter().unique().count() == word.len())
//         .collect::<Vec<&str>>()
//         .join("\n");
//     fs::write("sgb-words-trimmed.txt", words)
// }
