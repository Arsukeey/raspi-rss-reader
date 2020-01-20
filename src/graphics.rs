use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use std::collections::HashSet;
use std::path::Path;

use crate::items::{unify, News, RSS};

mod scr {
    pub const WIDTH: u32 = 800;
    pub const HEIGHT: u32 = 480;
}

mod buttons {
    pub const WIDTH: i32 = (800 - 60);
    pub const UP_H: i32 = 300;
    pub const DOWN_H: i32 = 360;
    pub const RELOAD_H: i32 = 420;
    pub const SIZE: i32 = 50;
}

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub struct Renderer {
    pub news: Vec<News>,
    pub canvas: Canvas<Window>,
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub _image_context: sdl2::image::Sdl2ImageContext,
    pub texture_creator: TextureCreator<WindowContext>,
    pub ttf_context: sdl2::ttf::Sdl2TtfContext,
    pub event_pump: sdl2::EventPump,
    pub news_index: usize,
    pub showing_desc: bool,
    pub selected_news: usize,
}

impl Renderer {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

        // load rss
        let mut g1news = RSS::default();
        g1news.refresh_g1()?;

        let mut sputnik = RSS::default();
        sputnik.refresh_sputnikbr()?;

        /*
        let mut bbc = RSS::default();
        bbc.refresh_bbc()?;
        */

        // group all the rss feeds in a single vector
        let news = unify(vec![g1news.items, sputnik.items]);
        // news.shuffle(&mut thread_rng());

        let window = video_subsystem
            .window("Raspberry Pi RSS Reader", scr::WIDTH, scr::HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        let event_pump = sdl_context.event_pump()?;
        canvas.set_draw_color(Color::RGB(0xE9, 0xE9, 0xE9));

        Ok(Self {
            news,
            canvas,
            sdl_context,
            video_subsystem,
            _image_context,
            texture_creator,
            ttf_context,
            event_pump,
            news_index: 0,
            showing_desc: false,
            selected_news: 0,
        })
    }

    pub fn prepare(&mut self) -> Result<(), String> {
        self.showing_desc = false;

        let image_not_avail = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/no_image.jpg"))?;

        let arrow_up = self.texture_creator.load_texture(Path::new("/usr/share/raspi-rss-reader/up.png"))?;
        let arrow_down = self.texture_creator.load_texture(Path::new("/usr/share/raspi-rss-reader/down.png"))?;
        let reload = self.texture_creator.load_texture(Path::new("/usr/share/raspi-rss-reader/reload.png"))?;

        let font = self.ttf_context.load_font("/usr/share/raspi-rss-reader/helvetica.ttf", 128)?;

        for (i, item) in self.news[self.news_index..(self.news_index + 3)]
            .iter()
            .enumerate()
        {
            let y = i as u32 * (scr::HEIGHT / 4 + 40) + 10;

            if let Some(image) = &item.image {
                let img = self.texture_creator.load_texture(Path::new(&image))?;
                self.canvas.copy(&img, None, rect!(10, y, 120, 130))?;
            } else {
                self.canvas
                    .copy(&image_not_avail, None, rect!(10, y, 120, 130))?;
            }

            const BASE: usize = 63;

            let text = item.title.to_owned();

            let dx;
            if i == 2 {
                dx = scr::WIDTH - 150 - 60; // adapt to buttons
            } else {
                dx = scr::WIDTH - 150;
            }

            let dy;
            if text.len() < BASE {
                dy = 45;
            } else {
                dy = 90;
            }

            let surface = font
                .render(&text)
                .blended_wrapped(Color::RGB(0, 0, 0), 4000)
                .map_err(|e| e.to_string())?;
            let text = self
                .texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;

            self.canvas.copy(&text, None, rect!(140, y + 10, dx, dy))?;
        }

        // copied later to be in the "last layer"
        self.canvas.copy(
            &arrow_up,
            None,
            rect!(buttons::WIDTH, buttons::UP_H, buttons::SIZE, buttons::SIZE),
        )?;

        self.canvas.copy(
            &arrow_down,
            None,
            rect!(
                buttons::WIDTH,
                buttons::DOWN_H,
                buttons::SIZE,
                buttons::SIZE
            ),
        )?;

        self.canvas.copy(
            &reload,
            None,
            rect!(
                buttons::WIDTH,
                buttons::RELOAD_H,
                buttons::SIZE,
                buttons::SIZE
            ),
        )?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), String> {
        self.prepare()?;
        let mut prev_buttons = HashSet::new();

        self.canvas.present();
        'running: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::Quit { .. } => break 'running,
                    _ => {}
                }
            }

            let mouse_state = self.event_pump.mouse_state();
            let buttons: HashSet<_> = mouse_state.pressed_mouse_buttons().collect();

            let new_buttons = &buttons - &prev_buttons;

            if !new_buttons.is_empty() {
                self.handle_mouse_state(mouse_state.x(), mouse_state.y())?;
            }

            prev_buttons = buttons;

            self.canvas.clear();

            if !self.showing_desc {
                self.prepare()?;
            } else {
                self.show_description(self.news[self.selected_news].clone())?;
            }
            self.canvas.present();
        }
        Ok(())
    }

    fn show_description(&mut self, news: News) -> Result<(), String> {
        self.showing_desc = true;

        let image_not_avail = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/no_image.jpg"))?;

        let img;
        if let Some(image) = &news.image {
            img = self.texture_creator.load_texture(Path::new(&image))?;
        } else {
            img = image_not_avail;
        }

        let ret = self.texture_creator.load_texture(Path::new("/usr/share/raspi-rss-reader/return.png"))?;
        let font = self.ttf_context.load_font("/usr/share/raspi-rss-reader/helvetica.ttf", 128)?;

        let dy;
        let t = news.title;

        if t.len() < 65 {
            dy = 50;
        } else {
            dy = 100;
        }

        let surface = font
            .render(&t)
            .blended_wrapped(Color::RGB(0, 0, 0), 4000)
            .map_err(|e| e.to_string())?;
        let title = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let mut d = news.desc;
        if d.len() > 1000 {
            d.drain(1000..);
            d.push_str("...");
        }
        let surface = font
            .render(&d)
            .blended_wrapped(Color::RGB(0, 0, 0), 4000)
            .map_err(|e| e.to_string())?;
        let desc = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        self.canvas.copy(&img, None, rect!(10, 10, 120, 130))?;
        self.canvas.copy(&title, None, rect!(140, 10, 650, dy))?;
        self.canvas
            .copy(&desc, None, rect!(10, 150, 750, scr::HEIGHT - 160))?;
        self.canvas.copy(&ret, None, rect!(720, 410, 70, 70))?;

        Ok(())
    }

    #[inline]
    fn scroll_up(&mut self) {
        eprintln!("Scrolling up...");
        if self.news_index > 0 {
            self.news_index -= 1;
        }
    }

    #[inline]
    fn scroll_down(&mut self) {
        eprintln!("Scrolling down...");
        if self.news_index < self.news.len() - 3 {
            self.news_index += 1;
        }
    }

    fn refresh(&mut self) -> Result<(), String> {
        eprintln!("Refreshing...");
        let mut g1news = RSS::default();
        g1news.refresh_g1()?;

        let mut sputnik = RSS::default();
        sputnik.refresh_sputnikbr()?;

        /*
        let mut bbc = RSS::default();
        bbc.refresh_bbc()?;
        */

        let n = unify(vec![g1news.items, sputnik.items]);

        self.news = n;

        Ok(())
    }

    fn handle_mouse_state(&mut self, x: i32, y: i32) -> Result<(), String> {
        if x >= buttons::WIDTH && y >= buttons::UP_H {
            if self.showing_desc {
                if y >= 410 {
                    self.prepare()?;
                }
            } else {
                if y < buttons::DOWN_H {
                    self.scroll_up();
                    self.prepare()?;
                } else if y < buttons::RELOAD_H {
                    self.scroll_down();
                    self.prepare()?;
                } else {
                    self.refresh()?;
                }
            }
        } else {
            self.selected_news = self.news_index + ((y - 1) / 160) as usize;
            self.show_description(self.news[self.selected_news].clone())?;
        }
        Ok(())
    }
}
