// TODO: add ssh support to open news on browser

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use std::cell::RefCell;

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
    pub news: RefCell<Vec<News>>,
    pub canvas: RefCell<Canvas<Window>>,
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub _image_context: sdl2::image::Sdl2ImageContext,
    pub texture_creator: TextureCreator<WindowContext>,
    pub ttf_context: sdl2::ttf::Sdl2TtfContext,
    pub event_pump: RefCell<sdl2::EventPump>,
    pub news_index: RefCell<usize>,
    pub showing_desc: RefCell<bool>,
    pub selected_news: RefCell<usize>,
}

struct Textures<'a, 'b> {
    pub image_not_avail: Texture<'a>,
    pub arrow_up: Texture<'a>,
    pub arrow_down: Texture<'a>,
    pub reload: Texture<'a>,
    pub go_back: Texture<'a>,
    pub font: Font<'a, 'b>,
    pub images: RefCell<Vec<Texture<'a>>>,
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
        let news = RefCell::new(unify(vec![g1news.items, sputnik.items]));
        // news.shuffle(&mut thread_rng());

        let window = video_subsystem
            .window("Raspberry Pi RSS Reader", scr::WIDTH, scr::HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = RefCell::new(window.into_canvas().build().map_err(|e| e.to_string())?);
        let texture_creator = canvas.borrow_mut().texture_creator();

        let event_pump = RefCell::new(sdl_context.event_pump()?);
        canvas
            .borrow_mut()
            .set_draw_color(Color::RGB(0xE9, 0xE9, 0xE9));

        Ok(Self {
            news,
            canvas,
            sdl_context,
            video_subsystem,
            _image_context,
            texture_creator,
            ttf_context,
            event_pump,
            news_index: RefCell::new(0),
            showing_desc: RefCell::new(false),
            selected_news: RefCell::new(0),
        })
    }

    fn prepare(&self, textures: &Textures) -> Result<(), String> {
        *self.showing_desc.borrow_mut() = false;

        let arrow_up = &textures.arrow_up;
        let arrow_down = &textures.arrow_down;
        let reload = &textures.reload;

        let font = &textures.font;

        const BASE: usize = 63;

        for (i, item) in self.news.borrow()
            [(*self.news_index.borrow())..((*self.news_index.borrow()) + 3)]
            .iter()
            .enumerate()
        {
            let y = i as u32 * (scr::HEIGHT / 4 + 40) + 10;
            let text = &item.title;

            self.canvas.borrow_mut().copy(
                &textures.images.borrow()[*self.news_index.borrow() + i],
                None,
                rect!(10, y, 120, 130),
            )?;

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

            self.canvas
                .borrow_mut()
                .copy(&text, None, rect!(140, y + 10, dx, dy))?;
        }

        // copied later to be in the "last layer"
        self.canvas.borrow_mut().copy(
            &arrow_up,
            None,
            rect!(buttons::WIDTH, buttons::UP_H, buttons::SIZE, buttons::SIZE),
        )?;

        self.canvas.borrow_mut().copy(
            &arrow_down,
            None,
            rect!(
                buttons::WIDTH,
                buttons::DOWN_H,
                buttons::SIZE,
                buttons::SIZE
            ),
        )?;

        self.canvas.borrow_mut().copy(
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

    pub fn render(&self) -> Result<(), String> {
        let image_not_avail = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/no_image.jpg"))?;

        let arrow_up = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/up.png"))?;
        let arrow_down = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/down.png"))?;
        let reload = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/reload.png"))?;
        let go_back = self
            .texture_creator
            .load_texture(Path::new("/usr/share/raspi-rss-reader/return.png"))?;
        let font = self
            .ttf_context
            .load_font("/usr/share/raspi-rss-reader/helvetica.ttf", 128)?;

        let textures = Textures {
            image_not_avail,
            arrow_up,
            arrow_down,
            reload,
            font,
            go_back,
            images: RefCell::new(vec![]),
        };

        self.refresh(RefCell::new(&textures))?;
        self.prepare(&textures)?;
        self.canvas.borrow_mut().present();

        'running: loop {
            for event in self.event_pump.borrow_mut().wait_timeout_iter(100) {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::Quit { .. } => break 'running,
                    Event::MouseButtonDown { x: i, y: j, .. } => {
                        self.handle_mouse_state(&textures, i, j)?;

                        self.canvas.borrow_mut().clear();

                        if !*self.showing_desc.borrow() {
                            self.prepare(&textures)?;
                        } else {
                            self.show_description(
                                self.news.borrow()[*self.selected_news.borrow()].clone(),
                                &textures,
                            )?;
                        }

                        self.canvas.borrow_mut().present();
                    }
                    _ => {}
                }
            }
        }
        self.canvas.borrow_mut().present();
        Ok(())
    }

    fn show_description(&self, mut news: News, textures: &Textures) -> Result<(), String> {
        *self.showing_desc.borrow_mut() = true;

        let img = &textures.images.borrow()[self.selected_news.borrow().clone()];

        let ret = &textures.go_back;
        let font = &textures.font;

        let dy;
        let t = &news.title;

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

        let d = &mut news.desc;
        let dy_desc;
        if d.len() > 1000 {
            d.drain(997..);
            d.push_str("...");
        }

        if d.len() < 200 {
            dy_desc = scr::HEIGHT / 2 - 130;
        } else {
            dy_desc = scr::HEIGHT - 160;
        }

        let surface = font
            .render(&d)
            .blended_wrapped(Color::RGB(0, 0, 0), 4000)
            .map_err(|e| e.to_string())?;
        let desc = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        self.canvas
            .borrow_mut()
            .copy(&img, None, rect!(10, 10, 120, 130))?;
        self.canvas
            .borrow_mut()
            .copy(&title, None, rect!(140, 10, 650, dy))?;
        self.canvas
            .borrow_mut()
            .copy(&desc, None, rect!(10, 150, 750, dy_desc))?;
        self.canvas
            .borrow_mut()
            .copy(&ret, None, rect!(720, 410, 70, 70))?;

        Ok(())
    }

    fn scroll_up(&self) {
        eprintln!("Scrolling up...");
        let mut i = self.news_index.borrow_mut();
        if *i > 0 {
            *i -= 1;
        }
    }

    #[inline]
    fn scroll_down(&self) {
        eprintln!("Scrolling down...");
        let mut i = self.news_index.borrow_mut();
        if *i < self.news.borrow().len() - 3 {
            *i += 1;
        }
    }

    fn refresh<'a, 'b>(&'a self, textures: RefCell<&Textures<'a, 'b>>) -> Result<(), String> {
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

        *self.news.borrow_mut() = n;

        let texts = textures.borrow();
        let mut imgs = texts.images.borrow_mut();

        for item in self.news.borrow().iter() {
            if let Some(image) = &item.image {
                let img = self.texture_creator.load_texture(Path::new(&image))?;
                imgs.push(img);
            } else {
                let image_not_avail = self
                    .texture_creator
                    .load_texture(Path::new("/usr/share/raspi-rss-reader/no_image.jpg"))?;

                imgs.push(image_not_avail);
            }
        }

        Ok(())
    }

    fn handle_mouse_state<'a, 'b>(
        &'a self,
        textures: &Textures<'a, 'b>,
        x: i32,
        y: i32,
    ) -> Result<(), String> {
        if x >= buttons::WIDTH && y >= buttons::UP_H {
            if *self.showing_desc.borrow() {
                if y >= 410 {
                    self.prepare(&textures)?;
                }
            } else {
                if y < buttons::DOWN_H {
                    self.scroll_up();
                    self.prepare(&textures)?;
                } else if y < buttons::RELOAD_H {
                    self.scroll_down();
                    self.prepare(&textures)?;
                } else {
                    self.refresh(RefCell::new(textures))?;
                }
            }
        } else {
            *self.selected_news.borrow_mut() = *self.news_index.borrow() + ((y - 1) / 160) as usize;
            self.show_description(
                self.news.borrow()[self.selected_news.borrow().clone()].clone(),
                &textures,
            )?;
        }
        Ok(())
    }
}
