use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, DATE, HOST, CONTENT_LENGTH},
    Client, Method, Response, StatusCode,
    Body,
};
use indicatif::{MultiProgress, ProgressBar};
use tokio;
use quick_xml::{Reader, events::{Event, BytesStart, BytesText, BytesEnd}, Writer};
use bytes::Bytes;

use std::{
    convert::{TryFrom, Into},
    time::SystemTime,
    fs::File,
    io::{BufWriter, BufReader},
    path,
    ffi::OsStr,
    vec,
    collections::HashMap,
    io::Cursor,
};


use crate::util::{self, url, Info, f_name, };
use crate::variable::{INFO, MULBAR};
use std::io::{Write, Read};
use reqwest::header::{CONTENT_TYPE, HeaderName};

// CompleteMultipartUpload
async fn complete_upload(verify: Vec<String>, path: String, upload_id: String) {
    let xml = util::create_xml(verify);
    println!("{}", xml);
    let date = util::date();
    let Info {
        bucket,
        access_key_id,
        access_key_scret,
        endpoint,
    } = INFO.get().unwrap();
    let host = format!("{}.{}", bucket, endpoint);
    let auth = util::authorize(
        Method::POST,
        None,
        None,
        date.clone(),
        None,
        format!("/{}{}?uploadId={}", bucket, path, upload_id),
        access_key_id.clone(),
        access_key_scret.clone(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(DATE, HeaderValue::try_from(date).unwrap());
    headers.insert(HOST, HeaderValue::try_from(host.clone()).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::try_from(auth).unwrap());

    let supplement = format!("?uploadId={}", upload_id);
    let mut url = util::url(path.as_str());
    url.push_str(supplement.as_str());
    let req = Client::new()
        .post(&url)
        .headers(headers)
        .body(Vec::from(xml));
    let res = req
        .send()
        .await
        .unwrap();
    println!("{:?}", res.text().await.unwrap());
}
// InitiateMultipartUpload
async fn init_mul_upload(path: String) -> Result<String, ()> {
    let date = util::date();
    let Info {
        bucket,
        access_key_id,
        access_key_scret,
        endpoint,
    } = INFO.get().unwrap();
    let host = format!("{}.{}", bucket, endpoint);
    let auth = util::authorize(
        Method::POST,
        None,
        None,
        date.clone(),
        None,
        format!("/{}{}?uploads", bucket, path),
        access_key_id.clone(),
        access_key_scret.clone(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(DATE, HeaderValue::try_from(date).unwrap());
    headers.insert(HOST, HeaderValue::try_from(host.clone()).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::try_from(auth).unwrap());

    let mut url = util::url(path.as_str());
    url.push_str("?uploads");
    let req = Client::new()
        .post(&url)
        .headers(headers);
    let res = req.send().await.unwrap();
    let body = res.text().await.unwrap();

    let mut reader = Reader::from_str(body.as_str());
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"UploadId" => {
                let id = reader.read_text("UploadId", &mut Vec::new()).unwrap();
                return Ok(id)
            },
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    Err(())
}
// UploadPart
async fn upload_part(buf: Vec<u8>, upload_id: String, part_num: i32, path: String) -> String {
    let length = buf.len();
    let date = util::date();
    let Info {
        bucket,
        access_key_id,
        access_key_scret,
        endpoint,
    } = INFO.get().unwrap();
    let host = format!("{}.{}", bucket, endpoint);
    let auth = util::authorize(
        Method::PUT,
        None,
        None,
        date.clone(),
        None,
        format!("/{}{}?partNumber={}&uploadId={}", bucket, path, part_num, upload_id),
        access_key_id.clone(),
        access_key_scret.clone(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(DATE, HeaderValue::try_from(date).unwrap());
    headers.insert(HOST, HeaderValue::try_from(host.clone()).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::try_from(auth).unwrap());
    headers.insert(CONTENT_LENGTH, HeaderValue::try_from(length.to_string()).unwrap());

    let supplement = format!("?partNumber={}&uploadId={}", part_num, upload_id);
    let mut url = util::url(path.as_str());
    url.push_str(supplement.as_str());
    //let buf = Bytes::from(buf).slice(..length);
    let res = Client::new()
        .put(&url)
        .headers(headers)
        .body(buf)
        .send()
        .await
        .unwrap();
    let res_header = res.headers();
    let etag = res_header.get("ETag").unwrap().to_str().unwrap().to_string();
    etag

}
// upload the object
pub async fn upload(src: String, dec: String) {
    println!("start");
    let mut f = File::open(f_name(src)).unwrap();
    let f_size = f.metadata().unwrap().len();
    //let bar = MULBAR.add(f_size);

    tokio::spawn(async move {
        let mut part_num = 0i32;
        let upload_id = init_mul_upload(dec.clone()).await.unwrap();
        let mut verify: Vec<String> = Vec::new();
        loop {
            let mut buf = vec![0u8; 1024*1024];
            let length = f.read(buf.as_mut()).unwrap();
            buf.resize(length,0);
            if length == 0 {break;}
            part_num = part_num + 1;
            let etag = upload_part(buf, upload_id.clone(), part_num, dec.clone()).await;
            verify.push(etag);
            //bar.inc(length as u64);
        }
        complete_upload(verify, dec.clone(), upload_id.clone()).await;
        //bar.finish_with_message("finish");
    }).await;


    //MULBAR.join();
}

// GetObject
pub async fn get(src: String, dec: Option<String>) {
    let date = util::date();
    let url = &util::url(src.as_str());
    let Info {
        bucket,
        access_key_id,
        access_key_scret,
        endpoint,
    } = INFO.get().unwrap();
    let host = format!("{}.{}", bucket, endpoint);
    let content_type = "application/octet-stream".to_string();
    let auth = util::authorize(
        Method::GET,
        None,
        None,
        date.clone(),
        None,
        format!("/{}{}", bucket, src),
        access_key_id.clone(),
        access_key_scret.clone(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(DATE, HeaderValue::try_from(date).unwrap());
    headers.insert(HOST, HeaderValue::try_from(host.clone()).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::try_from(auth).unwrap());

    let req = Client::new()
        .get(url)
        .headers(headers);
    let mut res = req.send().await.unwrap();
    let file_size = res.headers().get(CONTENT_LENGTH).unwrap().to_str().unwrap();
    let file_size = file_size.to_string().parse::<u64>().unwrap();

    let bar = MULBAR.add(file_size);

    let mut f = match dec {
        Some(p) => File::create(p).unwrap(),
        None => File::create(f_name(src)).unwrap(),
    };

    tokio::spawn(async move {
        while let Some(chunk) = res.chunk().await.unwrap() {
            f.write(chunk.as_ref()).unwrap();
            f.flush();
            bar.inc(chunk.len() as u64);
        }
        bar.finish_with_message("finish");
    });


    MULBAR.join();
}

// HeadObject
async fn head(path: String) -> Result<Response, ()> {
    let date = util::date();
    let Info {
        bucket,
        access_key_id,
        access_key_scret,
        endpoint,
    } = INFO.get().unwrap();
    let host = format!("{}.{}", bucket, endpoint);
    let auth = util::authorize(
        Method::HEAD,
        None,
        None,
        date.clone(),
        None,
        format!("/{}{}", bucket, path),
        access_key_id.clone(),
        access_key_scret.clone(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(DATE, HeaderValue::try_from(date).unwrap());
    headers.insert(HOST, HeaderValue::try_from(host.clone()).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::try_from(auth).unwrap());

    let req = Client::new()
        .head(&util::url(path.as_str()))
        .headers(headers);
    let res = req.send().await.unwrap();

    Ok(res)
}
#[tokio::test]
async fn test() {
    let mut f = File::open("demo.pdf").unwrap();
    let mut buf = vec![0u8; 1024*1024];
    loop {
        let n = f.read(buf.as_mut()).unwrap();
        if n == 0 {break;}
        println!("{}", n);
    }
    println!("finish");
}
