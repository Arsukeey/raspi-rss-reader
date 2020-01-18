use rss::Channel;
use rss::Item;

use std::fs;
use std::fs::File;
use std::io::copy;
use std::iter::FromIterator;

#[derive(Default, Debug)]
pub struct RSS {
    pub items: Vec<News>,
}

#[derive(Default, Debug, Clone)]
pub struct News {
    pub title: String,
    pub desc: String,
    pub image: Option<String>,
    pub url: String,
    pub author: String,
    pub downloaded: bool,
    pub source: String,
}

impl News {
    pub fn from(
        item: Item,
        image: Option<String>,
        downloaded: bool,
        source: &str,
    ) -> Result<Self, String> {
        let mut title = item
            .title()
            .ok_or("could not find news' title")?
            .to_string();

        if source == "G1" {
            title = title.split('(').next().unwrap_or(&title).to_string();
        }

        let desc = item
            .description()
            .ok_or("could not find news' description")?
            .to_string();
        let url = item.link().ok_or("could not find news' url")?.to_string();
        let author = item.author().unwrap_or("No author found.").to_string();
        let source = source.to_string();

        Ok(Self {
            title,
            desc,
            image,
            url,
            author,
            downloaded,
            source,
        })
    }
}

impl RSS {
    pub fn default() -> Self {
        let mut entries = fs::read_dir("/tmp")
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();

        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.
        entries.sort();
        let entries = entries
            .iter()
            .map(|x| x.to_str().unwrap())
            .collect::<Vec<&str>>();

        if !entries.contains(&"/tmp/raspi-pi-reader") {
            let _ = fs::create_dir("/tmp/raspi-pi-reader");
        }

        Self {
            ..Default::default()
        }
    }

    pub fn from(items: Vec<News>) -> Self {
        Self { items }
    }

    pub fn refresh_sputnikbr(&mut self) -> Result<(), String> {
        let channel = Channel::from_url("https://br.sputniknews.com/export/rss2/archive/index.xml")
            .ok()
            .ok_or("Could not find Sputnik's news.")?;

        let items = Vec::from_iter(channel.items()[..10].iter().cloned());

        for item in items {
            let mut image = None;
            if let Some(enclosure) = item.enclosure() {
                image = download(enclosure.url()).ok();
            }
            self.items
                .push(News::from(item.clone(), image, true, "Sputnik BR")?);
        }
        Ok(())
    }

    /*
    pub fn refresh_bbc(&mut self) -> Result<(), String> {
        let channel = Channel::from_url("http://feeds.bbci.co.uk/news/rss.xml")
            .ok()
            .ok_or("Could not find BBC's news.")?;

        let items = Vec::from_iter(channel.items()[..10].iter().cloned());

        for item in items {
            self.items
                .push(News::from(item.clone(), None, false, "BBC News")?);
        }
        Ok(())
    }*/

    pub fn refresh_g1(&mut self) -> Result<(), String> {
        let err = "Could not fetch G1's news.".to_string();

        let cr = Channel::from_url("http://g1.globo.com/dynamo/ciencia-e-saude/rss2.xml")
            .ok()
            .ok_or_else(|| err.clone())?;
        let economia = Channel::from_url("http://g1.globo.com/dynamo/economia/rss2.xml")
            .ok()
            .ok_or_else(|| err.clone())?;

        let unified = unify(vec![cr.items().to_vec(), economia.items().to_vec()]);

        let items = Vec::from_iter(unified[..10].iter().cloned());

        for item in items {
            let mut image;

            if let Some(media) = item.extensions().get("media") {
                let mut downloaded = false;
                let x = media.get("content").ok_or("no content found")?[0].attrs();

                if x.get("medium").ok_or("could not find medium of image")? == "image" {
                    image = Some(x.get("url").ok_or("couldnt find image's url")?.to_string());
                    image = download(
                        &image.unwrap_or_else(|| "could not find image's path".to_string()),
                    )
                    .ok();
                    downloaded = true;
                } else {
                    image = None;
                }

                self.items
                    .push(News::from(item.clone(), image, downloaded, "G1")?);
            } else {
                self.items
                    .push(News::from(item.clone(), None, false, "G1 Notícias")?);
            }
        }
        Ok(())
    }
}

pub fn unify<T: Clone>(vs: Vec<Vec<T>>) -> Vec<T> {
    let mut v = vec![];
    for i in 0..vs[0].len() {
        for j in &vs {
            v.push(j[i].clone());
        }
    }
    v
}

pub fn download(url: &str) -> Result<String, String> {
    let x: String = (*url
        .split('/')
        .collect::<Vec<&str>>()
        .last()
        .unwrap_or(&"couldnt find name of image"))
    .to_string();

    let name = format!("/tmp/raspi-pi-reader/{}", x);

    if std::path::Path::new(&name).exists() {
        return Ok(name);
    }

    let mut response = reqwest::get(url)
        .ok()
        .ok_or("error while downloading images")?;

    let mut dest = {
        File::create(name.clone())
            .ok()
            .ok_or_else(|| format!("error while creating {}", name))?
    };

    let _ = copy(&mut response, &mut dest);
    Ok(name)
}
