use std::collections::BTreeMap;
use std::path::Path;

use image;
use reqwest;
use serde::Deserialize;
use serde_json;
use tokio;

const WIDTH: u32 = 794;
const HEIGHT: u32 = 1122;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data<'a> {
    #[serde(borrow)]
    client_vars: ClientVars<'a>,
}

#[derive(Deserialize)]
struct ClientVars<'a> {
    #[serde(borrow)]
    collab_client_vars: CollabClientVars<'a>,
}

#[derive(Deserialize)]
struct CollabClientVars<'a> {
    #[serde(borrow)]
    apool: Apool<'a>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Apool<'a> {
    #[serde(borrow)]
    num_to_attrib: BTreeMap<usize, Vec<&'a str>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args().skip(1).next().unwrap();
    let data = std::fs::read_to_string(&file)?;
    let data = data.lines().last().unwrap();
    let data: Data = serde_json::from_str(&data)?;
    let images: Vec<&str> = data
        .client_vars
        .collab_client_vars
        .apool
        .num_to_attrib
        .into_iter()
        .map(|(_k, v)| v)
        .filter(|v| v.len() == 2)
        .filter(|v| v[0] == "img")
        .map(|v| v[1])
        .collect();
    let out_dir = Path::new("/tmp/");
    for (index, image_url) in images.into_iter().enumerate() {
        let id = image_url.split('/').skip(5).next().unwrap();
        let out = out_dir.join(format!("{}_{}", index, id));
        if !out.exists() {
            let buffer = reqwest::get(image_url).await?.bytes().await?;
            let mut img = image::load_from_memory(&buffer)?;
            let mut img = img
                .crop(90, 60, WIDTH - 90 * 2, HEIGHT - 60 - 40)
                .into_rgb();
            for pixel in img.pixels_mut() {
                if pixel.0[0] > 100 || pixel.0[1] > 100 || pixel.0[2] > 100 {
                    pixel.0 = [255, 255, 255];
                }
            }
            dbg!(&out);
            img.save_with_format(out, image::ImageFormat::Png)?;
        }
    }

    Ok(())
}
