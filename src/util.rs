use reqwest::Method;
use base64::encode;
use crypto::sha1::Sha1;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use serde_yaml;
use httpdate::fmt_http_date;
use indicatif::{ProgressStyle, MultiProgress, ProgressBar};
use once_cell::sync::OnceCell;
use quick_xml::{events::{Event, BytesStart, BytesText, BytesEnd}, Writer};

use std::{fs, time::SystemTime, path::Path, io::Cursor, str::FromStr};
use crate::error::{Result, Error};
use crate::variable::{CFG_PATH, INFO, MULBAR};


// Generate Authorization field
pub fn authorize(
    method: Method,
    content_md5: Option<String>,
    content_type: Option<String>,
    date: String,
    canonicalized_oss_headers: Option<String>,
    canonicalized_resource: String,
    access_key_id: String,
    access_key_scret: String
) -> String {

    let method = method.as_str();
    let content_md5 = content_md5.unwrap_or("".to_string());
    let content_type = content_type.unwrap_or("".to_string());
    let canonicalized_oss_headers = canonicalized_oss_headers.unwrap_or("".to_string());

    let sign_str = format!(
        "{}\n{}\n{}\n{}{}\n{}",
        method,
        content_md5,
        content_type,
        date,
        canonicalized_oss_headers,
        canonicalized_resource
    );
    let mut hasher = Hmac::new(Sha1::new(), access_key_scret.as_bytes());
    hasher.input(sign_str.as_bytes());
    let sign_str_base64 = encode(hasher.result().code());

    let authorization = format!(
        "OSS {}:{}",
        access_key_id,
        sign_str_base64
    );
    authorization
}

// 生成文件URL
pub fn url(path: &str) -> String {
    //https://t-cloud.oss-cn-hangzhou.aliyuncs.com/test/package.json
    let Info {endpoint, bucket, .. } = INFO.get().unwrap();
    format!("https://{}.{}{}", bucket, endpoint, path)
}
//生成GMT格式的date
pub fn date() -> String {fmt_http_date(SystemTime::now())}

// 初始化程序


#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename(deserialize = "EndPoint"))]
    pub endpoint: String,
    #[serde(rename(deserialize = "AccesskeyId"))]
    pub access_key_id: String,
    #[serde(rename(deserialize = "AccessKeyScret"))]
    pub access_key_scret: String,
    #[serde(rename(deserialize = "Bucket"))]
    pub bucket: String,
}

pub fn init() -> Result<()>{
    // 读取配置文件
    let cfg_content = String::from_utf8(fs::read(CFG_PATH)?).unwrap();
    let info: Info = serde_yaml::from_str(cfg_content.as_str()).unwrap();
    INFO.set(info);
    // 初始化进度条
    MULBAR.init();
    Ok(())
}


// 进度条
pub struct MulBar {
    pub mulbar: OnceCell<MultiProgress>,
}
impl MulBar {
    pub fn init(&self) {
        self.mulbar.set(MultiProgress::new());
    }
    // 添加进度条
    pub fn add(&self, size: u64) -> ProgressBar {
        let bar = self.mulbar.get().unwrap().add(ProgressBar::new(size));
        bar.set_style(self.create_style());
        bar
    }
    // 阻塞
    pub fn join(&self) {
        self.mulbar.get().unwrap().join().unwrap();
    }
    // 生成进度条样式
    pub fn create_style(&self) -> ProgressStyle {
        ProgressStyle::default_bar()
            .progress_chars("## ")
            .template("[{bar:40.white/blue}] {bytes}/{total_bytes}  {bytes_per_sec}  {eta}  {msg}")
    }
}

// 提取路径中的文件名
pub fn f_name(path: String) -> String {
    Path::new(&path).file_name().unwrap().to_str().unwrap().to_string()
}

// 生成CompleteMultipartUpload中的xml
pub fn create_xml(verify: Vec<String>) -> String {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let elem_1_name = "CompleteMultipartUpload";
    let elem_1_start = BytesStart::owned(elem_1_name.as_bytes(), elem_1_name.len());
    let elem_1_end = BytesEnd::owned(Vec::from(elem_1_name));
    let elem_2_start = BytesStart::owned("Part".as_bytes(), "Part".len());
    let elem_2_end = BytesEnd::owned(Vec::from("Part"));
    let elem_3_start = BytesStart::owned("PartNumber".as_bytes(), "PartNumber".len());
    let elem_3_end = BytesEnd::owned(Vec::from("PartNumber"));
    let elem_4_start = BytesStart::owned("ETag".as_bytes(), "ETag".len());
    let elem_4_end = BytesEnd::owned(Vec::from("ETag"));

    writer.write_event(Event::Start(elem_1_start));
    let mut i = 0;
    for v in verify {
        i = i + 1;
        println!("verify:{},{}", i, v);
        writer.write_event(Event::Start(elem_2_start.to_owned()));
        writer.write_event(Event::Start(elem_3_start.to_owned()));
        let i = i.to_string();
        let num = BytesText::from_plain_str(i.as_str());
        writer.write_event(Event::Text(num));
        writer.write_event(Event::End(elem_3_end.to_owned()));
        writer.write_event(Event::Start(elem_4_start.to_owned()));
        let tag = BytesText::from_plain_str(v.as_str());
        writer.write_event(Event::Text(tag));
        writer.write_event(Event::End(elem_4_end.to_owned()));
        writer.write_event(Event::End(elem_2_end.to_owned()));
    }
    writer.write_event(Event::End(elem_1_end));

    String::from_utf8(writer.into_inner().into_inner().to_vec()).unwrap()
}
