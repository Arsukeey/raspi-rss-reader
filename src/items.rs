use rss::Channel;
use rss::Item;

#[derive(Default, Debug)]
pub struct RSS {
    items: Vec<News>,
    source: String,
}

#[derive(Default, Debug)]
struct News {
    title: String,
    desc: String,
    image: Option<String>,
    url: String,
    author: String,
}

impl News {
    pub fn from(item: Item, image: Option<String>) -> Result<Self, String> {
        let title = item
            .title()
            .ok_or("could not find news' title")?
            .to_string();
        let desc = item
            .description()
            .ok_or("could not find news' description")?
            .to_string();
        let url = item.link().ok_or("could not find news' url")?.to_string();
        let author = item.author().unwrap_or("No author found.").to_string();

        Ok(Self {
            title,
            desc,
            image,
            url,
            author,
        })
    }
}

impl RSS {
    pub fn default() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn refresh_sputnikbr(&mut self) -> Result<(), String> {
        let channel = Channel::from_url("https://br.sputniknews.com/export/rss2/archive/index.xml")
            .ok()
            .ok_or("Could not find Sputnik's news.")?;

        let items = channel.items();

        for item in items {
            let image = Some(
                item.enclosure()
                    .ok_or("couldn't find image's enclosure")?
                    .url()
                    .to_string(),
            );
            self.items.push(News::from(item.clone(), image)?);
        }
        self.source = "Sputnik BR".to_string();
        Ok(())
    }

    pub fn refresh_g1(&mut self) -> Result<(), String> {
        let err = "Could not fetch G1's news.".to_string();

        let cr = Channel::from_url("http://g1.globo.com/dynamo/ciencia-e-saude/rss2.xml")
            .ok()
            .ok_or_else(|| err.clone())?;
        let economia = Channel::from_url("http://g1.globo.com/dynamo/economia/rss2.xml")
            .ok()
            .ok_or_else(|| err.clone())?;
        let tech = Channel::from_url("http://g1.globo.com/dynamo/tecnologia/rss2.xml")
            .ok()
            .ok_or_else(|| err.clone())?;
        let sp = Channel::from_url("http://g1.globo.com/dynamo/sao-paulo/rss2.xml")
            .ok()
            .ok_or_else(|| err.clone())?;

        let items = unify(vec![
            cr.items().to_vec(),
            economia.items().to_vec(),
            tech.items().to_vec(),
            sp.items().to_vec(),
        ]);

        for item in items {
            let mut image = None;
            let x = item
                .extensions()
                .get("media")
                .ok_or("no media found")?
                .get("content")
                .ok_or("no content found")?[0]
                .attrs();

            if x.get("medium").ok_or("medium was not an image")? == "image" {
                image = Some(x.get("url").ok_or("couldnt find image's url")?.to_string());
            }

            self.items.push(News::from(item.clone(), image)?);
        }
        self.source = "G1".to_string();
        Ok(())
    }
}

fn unify(vs: Vec<Vec<Item>>) -> Vec<Item> {
    let mut v = vec![];
    for i in 0..vs[0].len() {
        for j in &vs {
            v.push(j[i].clone());
        }
    }
    v
}
