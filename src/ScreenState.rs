use crate::event_loop::EventLoop;
use crate::GameData::GameData;
use femtovg::renderer::OpenGl;
use femtovg::{renderer::Renderer, Align, Baseline, Canvas, Color, Paint, Path};
use glutin::dpi::PhysicalSize;
use glutin::window::Window;
use glutin::PossiblyCurrent;
pub use glutin::{event_loop, window::WindowBuilder, ContextBuilder, ContextWrapper};
use std::ops::{Deref, DerefMut};
use std::time::Instant;

pub(crate) trait ScreenElement<R: Renderer> {
    //type R: Renderer;
    fn render(&self, canvas: &mut Canvas<R>);
}

pub(crate) struct ScreenData<'a, R: Renderer> {
    pub(crate) screen_elements: Vec<Box<dyn ScreenElement<R> + 'a>>,
    pub(crate) prevt: Instant,
    pub(crate) perf: PerfGraph,
    pub(crate) canvas: Canvas<R>,
}

impl<'a, R: Renderer> ScreenData<'_, R> {
    pub(crate) fn init(canvas: Canvas<R>) -> ScreenData<'a, R> {
        //Attach Elements from Game Data
        ScreenData {
            screen_elements: vec![],
            prevt: Instant::now(),
            perf: PerfGraph::new(),
            canvas,
        }
    }

    pub(crate) fn init_frame(&mut self, game_data: &GameData) {
        self.screen_elements.clear();
        for row in &game_data.attempt_letter_blocks {
            for block in row {
                self.screen_elements.push(Box::new(*block));
            }
        }
        for block in &game_data.keyboard_letter_blocks {
            self.screen_elements.push(Box::new(*block));
        }
    }
}

// impl<'a, R: Renderer> Deref for ScreenData<'_, R> {
//     type Target = Vec<Box<dyn ScreenElement<R>>>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.screen_elements
//     }
// }

// impl<R: Renderer> DerefMut for ScreenData<'_, R> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.screen_elements
//     }
// }

pub(crate) struct PerfGraph {
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

    pub(crate) fn update(&mut self, frame_time: f32) {
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
