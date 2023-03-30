use chrono::Local;
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

#[derive(Deserialize)]
struct Data {
    PName: String, // 品名
    LPrice: String, // 最低价
    MPrice: String,
    PPrice: String,
    PSort: String,
    ReleaseTime: String,
    Standard: String,
}

async fn fetch_data(
    page: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    // reltime的值: 蔬菜 水产品 果品 副食品
    let params = [
        ("pageNum", page.to_string()),
        ("pname", "".to_owned()),
        ("reltime", "蔬菜".to_owned()),
    ];
    let res = client
        .post("http://www.cncyms.cn/pages.php")
        .form(&params)
        .send()
        .await?;
    let res_text = res.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&res_text).unwrap();
    let data: Vec<Data> = serde_json::from_value::<Vec<Data>>(json["list"].clone())
        .expect("invalid type: null, expected a sequence");
    let file_name = Local::now().format("%Y-%m-%d.csv").to_string();
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name)?;
    let mut file = std::io::BufWriter::new(file);
    data.iter().for_each(|d| {
        if d.ReleaseTime != chrono::Local::now().format("%Y-%m-%d").to_string() {
            panic!("Not current date");
        }
        writeln!(
            file,
            "{},{},{},{},{},{},{}",
            d.PSort, d.PName, d.ReleaseTime, d.PPrice, d.LPrice, d.MPrice, d.Standard
        )
        .unwrap();
    });
    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let file_name = Local::now().format("%Y-%m-%d.csv").to_string();
    let mut file = File::create(file_name)?;
    writeln!(file, "品种,品名,发布日期,中间参考价,最低价,最高价,规格")?;
    drop(file);
    for page in 1.. {
        println!("{}", page);
        if fetch_data(page, &client).await.is_err() {
            break;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures::executor::block_on(run())?;
    Ok(())
}
