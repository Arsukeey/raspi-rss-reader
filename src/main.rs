extern crate reqwest;
extern crate rss;
extern crate sdl2;

pub mod graphics;
pub mod items;

fn main() -> Result<(), String> {
    let mut g1 = crate::items::RSS::default();
    g1.refresh_g1()?;

    let mut sp = crate::items::RSS::default();
    sp.refresh_sputnikbr()?;

    println!("{:#?}", sp.items[0]);
    println!("{:#?}", g1.items[0]);

    Ok(())
}
