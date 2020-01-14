use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadSurface};
use sdl2::pixels::Color;
use std::time::Duration;

use crate::items::RSS;

pub fn render() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    let mut g1news = RSS::default();
    g1news.refresh_g1()?;

    let mut sputnik = RSS::default();
    sputnik.refresh_sputnikbr()?;

    let window = video_subsystem
        .window("RSS Reader", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    println!("{:?}", sputnik.items[0].title);

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}
