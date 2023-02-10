#![deny(elided_lifetimes_in_paths)]

use std::borrow::BorrowMut;
//use std::intrinsics::size_of;
use crate::event_loop::{ControlFlow, EventLoopWindowTarget};
pub use femtovg::{
    renderer::OpenGl, renderer::Renderer, renderer::Void, Align, Baseline, Canvas, Color, Paint,
    Path,
};
use glutin::event::Event;
pub use glutin::{event_loop, window::WindowBuilder, ContextBuilder, ContextWrapper};
use std::env;
use std::error::Error;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::time::Instant;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref GREY: Paint = Paint::color(Color::hex("999999"));
    pub static ref YELLOW: Paint = Paint::color(Color::hex("FFFF00"));
    pub static ref GREEN: Paint = Paint::color(Color::hex("00FF00"));
    pub static ref BLACK: Paint = Paint::color(Color::hex("000000"));
    pub static ref WHITE: Paint = Paint::color(Color::hex("FFFFFF"));
    pub static ref DEFAULT_LETTER_BLOCK: LetterBlock = LetterBlock {
        letter: 'c',
        fill_color: *GREY,
        width: 50.0,
        height: 50.0,
        x: 0.0,
        y: 0.0
    };
}

pub fn one_over(prev: &LetterBlock) -> f32 {
    let space = 5.0;
    prev.x + prev.width + space
}
pub fn one_below(prev: &LetterBlock) -> f32 {
    let space = 5.0;
    prev.y + prev.height + space
}

fn main() {
    //Read in word list
    let words = fs::read_to_string("sgb-words.txt").unwrap().split("\n");

    //Instantiate OpenGl Context
    let window_size = glutin::dpi::PhysicalSize::new(1000, 1000);
    let el = event_loop::EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_title("Gradient Test");
    let window_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let window_context = unsafe { window_context.make_current().unwrap() };

    //Create Rendering and Canvas Objects
    let render_obj = OpenGl::new_from_glutin_context(&window_context)
        .ok()
        .unwrap();
    let mut canvas = Canvas::new(render_obj).expect("Cannot create canvas");
    canvas.add_font("SpaceGrotesk-Bold.ttf");
    canvas.set_size(
        window_size.width as u32,
        window_size.height as u32,
        window_context.window().scale_factor() as f32,
    );

    let start = Instant::now();
    let mut prevt = start;
    let mut perf = PerfGraph::new();

    let row1: LetterBlockRow<'static> = LetterBlockRow::init("     ", 0.0, 0.0, 5.0);
    let row2: LetterBlockRow<'static> =
        LetterBlockRow::init("     ", 0.0, one_below(&row1.letters[0]), 5.0);
    let row3: LetterBlockRow<'static> =
        LetterBlockRow::init("     ", 0.0, one_below(&row2.letters[0]), 5.0);
    let row4: LetterBlockRow<'static> =
        LetterBlockRow::init("     ", 0.0, one_below(&row3.letters[0]), 5.0);
    let row5: LetterBlockRow<'static> =
        LetterBlockRow::init("     ", 0.0, one_below(&row4.letters[0]), 5.0);
    let keyboard1: LetterBlockRow<'static> =
        LetterBlockRow::init("QWERTYUIOP", 0.0, 100.0 + one_below(&row5.letters[0]), 5.0);
    let keyboard2: LetterBlockRow<'static> =
        LetterBlockRow::init("ASDFGHJKL", 0.0, one_below(&keyboard1.letters[0]), 5.0);
    let keyboard3: LetterBlockRow<'static> =
        LetterBlockRow::init("ZXCVBNM", 0.0, one_below(&keyboard2.letters[0]), 5.0);
    let mut rows = vec![
        Box::from(row1),
        Box::from(row2),
        Box::from(row3),
        Box::from(row4),
        Box::from(row5),
        Box::from(keyboard1),
        Box::from(keyboard2),
        Box::from(keyboard3),
    ];

    let mut screen_data: ScreenData<'static> = ScreenData { elements: vec![] };
    screen_data.elements.append(&mut rows);

    el.run(move |event: Event<'_, ()>, _, control_flow| {
        *control_flow = event_loop::ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                for element in screen_data.elements.iter() {
                    element.render(&mut canvas);
                }
                let now = Instant::now();
                let dt = (now - prevt).as_secs_f32();
                prevt = now;
                perf.update(dt);

                canvas.save();
                canvas.reset();
                //perf.render(&mut canvas, 5.0, 5.0);
                canvas.restore();
                canvas.flush();
                window_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

//#[derive(Derivative)]
//#[derivative(Debug, Default)]
pub struct LetterBlock {
    letter: char,
    fill_color: Paint,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
}

impl LetterBlock {
    fn render<T: Renderer>(&self, canvas: &mut Canvas<T>) {
        let mut path = Path::new();
        path.rounded_rect(self.x, self.y, self.width, self.height, 5.0);
        canvas.fill_path(&mut path, self.fill_color);
        canvas.stroke_path(&mut path, self.fill_color);

        let middle_x = self.x + self.width / 2.0;
        let middle_y = self.y + self.height / 2.0;
        let mut text_paint = *WHITE;
        //text_paint.set_font();
        text_paint.set_font_size(36.0);
        text_paint.set_text_align(Align::Center);
        text_paint.set_text_baseline(Baseline::Middle);
        let _ = canvas
            .fill_text(middle_x, middle_y, self.letter.to_string(), text_paint)
            .expect("Text Render ERROR");
    }
}

struct LetterBlockRow<'a> {
    as_string: &'a str,
    start_x: f32,
    start_y: f32,
    spacing: f32,
    letters: Vec<LetterBlock>,
}

impl LetterBlockRow<'_> {
    fn init(as_string: &str, start_x: f32, start_y: f32, spacing: f32) -> LetterBlockRow<'_> {
        let mut letter_arr: Vec<LetterBlock> = Vec::new();
        for (i, c) in as_string.chars().enumerate() {
            letter_arr.push(LetterBlock {
                letter: c,
                x: start_x + (i as f32) * DEFAULT_LETTER_BLOCK.width + ((i as f32) - 1.0) * spacing,
                y: start_y,
                ..*DEFAULT_LETTER_BLOCK
            });
        }
        LetterBlockRow {
            as_string,
            start_x,
            start_y,
            spacing,
            letters: letter_arr,
        }
    }
}

impl ScreenElement for LetterBlockRow<'_> {
    type R = OpenGl;
    fn render<T: Renderer>(&self, canvas: &mut Canvas<T>) {
        for letter_block in &self.letters {
            letter_block.render(canvas);
        }
    }
}

struct ScreenData<'a> {
    elements: Vec<Box<LetterBlockRow<'a>>>,
}

impl<'a> Deref for ScreenData<'a> {
    type Target = Vec<Box<LetterBlockRow<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<'a> DerefMut for ScreenData<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

trait ScreenElement {
    type R: Renderer;
    fn render<R: Renderer>(&self, canvas: &mut Canvas<R>);
}

struct PerfGraph {
    history_count: usize,
    values: Vec<f32>,
    head: usize,
}

impl PerfGraph {
    fn new() -> Self {
        Self {
            history_count: 100,
            values: vec![0.0; 100],
            head: Default::default(),
        }
    }

    fn update(&mut self, frame_time: f32) {
        self.head = (self.head + 1) % self.history_count;
        self.values[self.head] = frame_time;
    }

    fn get_average(&self) -> f32 {
        self.values.iter().map(|v| *v).sum::<f32>() / self.history_count as f32
    }

    fn render<T: Renderer>(&self, canvas: &mut Canvas<T>, x: f32, y: f32) {
        let avg = self.get_average();

        let w = 200.0;
        let h = 35.0;

        let mut path = Path::new();
        path.rect(x, y, w, h);
        canvas.fill_path(&mut path, Paint::color(Color::rgba(0, 0, 0, 128)));

        //let middle_x = self.x + self.width/2.0;
        //let middle_y = self.y + self.height/2.0;
        //let mut text_paint = WHITE;
        //text_paint.set_font_size(48.0);
        // text_paint.set_text_align(Align::Center);
        // text_paint.set_text_baseline(Baseline::Middle);
        // canvas.fill_text(middle_x, middle_y, self.letter, text_paint);

        let mut path = Path::new();
        path.move_to(x, y + h);

        for i in 0..self.history_count {
            let mut v = 1.0 / (0.00001 + self.values[(self.head + i) % self.history_count]);
            if v > 80.0 {
                v = 80.0;
            }
            let vx = x + (i as f32 / (self.history_count - 1) as f32) * w;
            let vy = y + h - ((v / 80.0) * h);
            path.line_to(vx, vy);
        }

        path.line_to(x + w, y + h);
        canvas.fill_path(&mut path, Paint::color(Color::rgba(255, 192, 0, 128)));

        let mut text_paint = Paint::color(Color::rgba(240, 240, 240, 255));
        text_paint.set_font_size(12.0);
        let _ = canvas.fill_text(x + 5.0, y + 13.0, "Frame time", text_paint);

        let mut text_paint = Paint::color(Color::rgba(240, 240, 240, 255));
        text_paint.set_font_size(14.0);
        text_paint.set_text_align(Align::Right);
        text_paint.set_text_baseline(Baseline::Top);
        let _ = canvas.fill_text(x + w - 5.0, y, &format!("{:.2} FPS", 1.0 / avg), text_paint);

        let mut text_paint = Paint::color(Color::rgba(240, 240, 240, 200));
        text_paint.set_font_size(12.0);
        text_paint.set_text_align(Align::Right);
        text_paint.set_text_baseline(Baseline::Alphabetic);
        let _ = canvas.fill_text(
            x + w - 5.0,
            y + h - 5.0,
            &format!("{:.2} ms", avg * 1000.0),
            text_paint,
        );
    }
}
