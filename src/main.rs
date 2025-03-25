#[macro_use] extern crate rocket;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::fs::FileServer;
use rocket::fs::NamedFile;
use rocket::fs::relative;
use std::path::{Path, PathBuf};
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::Data;
use rocket::http::ContentType;
use rocket_multipart_form_data::{mime, MultipartFormDataOptions, MultipartFormData, MultipartFormDataField, Repetition};
use rocket::tokio::time::{sleep, Duration};
use mysql_async::{OptsBuilder, Conn, Opts};
use mysql_async::prelude::*;

// 屬性以 #[...] 或 #![...] 的形式出現
// #[derive(...)] 是一種特殊的屬性（attribute），用於自動實作 trait。
// Trait 類似於其他程式語言中的介面（interface）
#[derive(Serialize, Deserialize)] // 
#[serde(crate = "rocket::serde")]
struct Message {
    message: String,
}

#[derive(Deserialize, Debug, FromForm)]
struct FormData {
    emp_no: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct ScanReport {
    report_file: String,
    report_data: Vec<u8>,
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    let path: PathBuf = Path::new("static/index.html").to_path_buf();
    NamedFile::open(path).await.ok()
}

#[get("/message/<name>")]
fn message(name: String) -> String {
    format!("Hello, {}!", name)
}

#[get("/getReportList")]
async fn getReportList() -> String {
    let path: PathBuf = Path::new("static/index.html").to_path_buf();
    //NamedFile::open(path).await.ok()
    format!("Hello, {}!", path.display())
}

#[post("/getReport", data = "<form>")]
async fn getReport(form: Form<FormData>) -> String {

    let opts = Opts::from_url("mysql://wonderful:123456@localhost/fscli").unwrap();
    let mut conn = Conn::new(opts).await.unwrap();
    let loaded_report = format!("SELECT report_file, report_data FROM reports where emp_no = {}", form.emp_no)
        .with(())
        .map(&mut conn, |(report_file, report_data)| ScanReport { report_file, report_data })
        .await;
    
    /*let result = match loaded_report {
        Ok(report) => report,
        Err(e) => "Error"
    };*/
    drop(conn);
    let mut res_msg = String::from("Error");
    if let Ok(report) = loaded_report {
        //format!("report_file: {}", report.report_file)
        for r in report {
            println!("{}", r.report_file);
        }
        format!("OK")
    }
    else {
        format!("Error")
    }

}

// 處理fscliApp丟過來的F-Secure 報告(.html檔)
//#[post("/uploadReport", data = "<upload>")]
//fn uploadReport(upload: Form<Upload>) -> String {
#[post("/uploadReport", data = "<data>")]
async fn uploadReport(content_type: &ContentType, data: Data<'_>) -> &'static str {

    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            //MultipartFormDataField::file("scan_file").content_type_by_string(Some(mime::TEXT)).unwrap(),
            // file is application/octet-stream
            //MultipartFormDataField::file("photo").content_type_by_string(Some(mime::IMAGE_STAR)).unwrap(),
            MultipartFormDataField::file("scan_file").content_type_by_string(Some(mime::APPLICATION_OCTET_STREAM)).unwrap(),
            MultipartFormDataField::text("result"),
            MultipartFormDataField::text("scan_file_name"),
            MultipartFormDataField::text("scan_time"),
            MultipartFormDataField::text("user"),
            MultipartFormDataField::text("report_file"),
        ]
    );
    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).await.unwrap();
    let user = multipart_form_data.texts.remove("user"); // Use the remove method to move text fields out of the MultipartFormData instance (recommended)
    /*let scan_file_path = multipart_form_data.texts.remove("scan_file");
    let scan_time = multipart_form_data.texts.remove("scan_time");
    let report_file = multipart_form_data.texts.remove("report_file");
    let result = multipart_form_data.texts.remove("result");
    let scan_file = multipart_form_data.files.remove("scan_file");*/
    // 取得前端上傳的檔案

    //println!("scan_file: {:?}", scan_file);
    let scan_file = multipart_form_data.files.get("scan_file");
    if let Some(file_fields) = scan_file {
        let file_field = &file_fields[0]; // Because we only put one "photo" field to the allowed_fields, the max length of this file_fields is 1.
     
        let _content_type = &file_field.content_type;
        let _file_name = &file_field.file_name;
        let _path = &file_field.path;
     
        // You can now deal with the uploaded file.
        println!("scan_file: {:?}", file_field.path);
    }   
    if let Some(mut text_fields) = user {
        println!("text_fields: {:?}", text_fields);
        let text_field = &text_fields[0]; //text_fields.remove(0); // Because we only put one "text" field to the allowed_fields, the max length of this text_fields is 1.
        //let _text = text_field.text;
        println!("user: {}", text_field.text);
    }
    
    "ok"
}

/*#[post("/json", format = "json", data = "<data>")]
fn post_json(data: Json<Message>) -> Json<Message> {
    //data
}*/

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, message, getReportList, getReport, uploadReport])
        .mount("/", FileServer::from(relative!("static"))) // 指定static目錄
}

/*use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use fsReportRust::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8089").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, filename) = match &request_line[..] { // [..] 表示取得整個字串的切片。
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "src/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(10));
            ("HTTP/1.1 200 OK", "src/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "src/404.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}*/

/*
// old version
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()    // 逐行讀取，並產生一個迭代器, 迭代器會產生 Result<String, io::Error>
        .map(|result| result.unwrap()) // |result| result.unwrap() 是一個閉包, map是迭代器轉換器，將資料透過閉包轉換
        .take_while(|line| !line.is_empty()) // take_while為迭代器轉換器，直到發現一個空白行為止
        .collect(); // 迭代器收集器，將所有元素收集到一個集合中(Vec<String>)。

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("hello.html").unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
*/