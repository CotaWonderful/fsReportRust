#[macro_use] extern crate rocket;
use rocket::serde::{Serialize, Deserialize, json::Json};
use serde_json::json;
use serde_json::Value;
use rocket::fs::FileServer;
use rocket::fs::NamedFile;
use rocket::fs::relative;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::Data;
use rocket::http::ContentType;
use rocket_multipart_form_data::{mime, MultipartFormDataOptions, MultipartFormData, MultipartFormDataField, Repetition};
use rocket::tokio::time::{sleep, Duration};
use mysql_async::{OptsBuilder, Conn, Opts};
use mysql_async::prelude::*;
use mysql_common::chrono::NaiveDateTime;

// 屬性以 #[...] 或 #![...] 的形式出現
// #[derive(...)] 是一種特殊的屬性（attribute），用於自動實作 trait。
// Trait 類似於其他程式語言中的介面（interface）
#[derive(Serialize, Deserialize)] // 
#[serde(crate = "rocket::serde")]
struct Message {
    message: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct ReportList {
    id: i32,
    scan_time: NaiveDateTime,
    scan_file: String,
}

#[derive(Deserialize, Debug, FromForm)]
struct FormData {
    emp_no: String,
    start_date: String,
    end_date: String,
}

#[get("/get_report_list")]
async fn get_report_list() -> Option<NamedFile> {
    let path: PathBuf = Path::new("static/index.html").to_path_buf();
    NamedFile::open(path).await.ok()
}

#[post("/get_report_list", data = "<form>")]
async fn get_report_list_post(form: Form<FormData>) -> Json<Vec<ReportList>> {

    let opts = Opts::from_url("mysql://wonderful:123456@localhost/fscli").unwrap();
    let mut conn = Conn::new(opts).await.unwrap();
    println!("form {:#?}", form);
    let sql =  format!("SELECT id FROM reports where emp_no = '{}' and scan_time between '{}' and '{}'", form.emp_no, form.start_date, form.end_date);
    println!("sql {:#?}", sql);
    // SELECT語法
    let loaded_reports = format!("SELECT id, scan_time, scan_file FROM reports where emp_no='{}' AND scan_time between '{}' AND '{}'", form.emp_no, form.start_date, form.end_date)
        .with(())
        .map(&mut conn, |(id, scan_time, scan_file)| ReportList { id, scan_time, scan_file })
        .await;
      
    println!("loaded_reports {:#?}", loaded_reports);
    
    //format!("OK")
    Json(loaded_reports.unwrap())
  
}


#[derive(Deserialize, Debug, FromForm)]
struct FormData2 {
    id: String,
}
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Report {
    report_file: String,
    report_data: Vec<u8>,
}
#[post("/get_report", data = "<form>")]
async fn get_report(form: Form<FormData2>) -> String {
    let opts = Opts::from_url("mysql://wonderful:123456@localhost/fscli").unwrap();
    let mut conn = Conn::new(opts).await.unwrap();
    // SELECT語法
    let loaded_reports = format!("SELECT report_file, report_data FROM reports where id='{}'", form.id)
        .with(())
        .map(&mut conn, |(report_file, report_data)| Report { report_file, report_data })
        .await;
    
    let mut report = loaded_reports.unwrap()[0].clone();
    let report_data_str = String::from_utf8(report.report_data).unwrap();
    println!("report_data_str {:#?}", report_data_str);
    let return_data = json!({
        "report_file": report.report_file,
        "report_data": report_data_str
    });
    //println!("loaded_reports {:#?}", loaded_reports.unwrap()[0].report_file);
    //Json(return_data)
    report_data_str
}

// 處理fscliApp丟過來的F-Secure 報告(.html檔)
//#[post("/uploadReport", data = "<upload>")]
//fn uploadReport(upload: Form<Upload>) -> String {
#[post("/upload_report", data = "<data>")]
async fn upload_report(content_type: &ContentType, data: Data<'_>) -> String {
    let mut error_message = String::from("");
    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::file("scan_file").content_type_by_string(Some(mime::APPLICATION_OCTET_STREAM)).unwrap(),
            MultipartFormDataField::text("result"),
            MultipartFormDataField::text("scan_file_name"),
            MultipartFormDataField::text("scan_time"),
            MultipartFormDataField::text("user"),
            MultipartFormDataField::text("report_file"),
        ]
    );
    
    loop {
        let mut multipart_form_data = match MultipartFormData::parse(content_type, data, options).await {
            Ok(multipart_form_data) => multipart_form_data,
            Err(e) => {
                error_message = format!("解析資料時發生錯誤: {}", e);
                break;
            }
        };
        // 解析欄位
        let user = match multipart_form_data.texts.remove("user") { // Use the remove method to move text fields out of the MultipartFormData instance (recommended)
            Some(user) => user[0].text.to_string(),
            None => {
                error_message = String::from("欄位不得為空: user");
                break;
            }
        };
        let scan_file_name = match multipart_form_data.texts.remove("scan_file_name") {
            Some(scan_file_name) => scan_file_name[0].text.to_string(),
            None => {
                error_message = String::from("欄位不得為空: scan_file_name");
                break;
            }
        };
        let scan_time = match multipart_form_data.texts.remove("scan_time") {
            Some(scan_time) => scan_time[0].text.to_string(),
            None => {
                error_message = String::from("欄位不得為空: scan_time");
                break;
            }
        };
        let report_file = match multipart_form_data.texts.remove("report_file") {
            Some(report_file) => report_file[0].text.to_string(),
            None => {
                error_message = String::from("欄位不得為空: report_file");
                break;
            }
        };
        let result = match multipart_form_data.texts.remove("result") {
            Some(result) => result[0].text.to_string(),
            None => {
                error_message = String::from("欄位不得為空: result");
                break;
            }
        };
        
        // 建立sql連線
        let opts = match Opts::from_url("mysql://wonderful:123456@localhost/fscli") {
            Ok(opts) => opts,
            Err(e) => {
                error_message = format!("Server端無法連接資料庫, 錯誤: {}", e);
                break;
            }
        };
        let mut conn = match Conn::new(opts).await {
            Ok(conn) => conn,
            Err(e) => {
                error_message = format!("Server端無法連接資料庫, 錯誤: {}", e);
                break;
            }
        };
        // 讀取檔案
        let file_fields_option = multipart_form_data.files.remove("scan_file");
        let file_path = match file_fields_option {
            Some(file_fields) => {
                //let file_field = &file_fields[0]; // Because we only put one "photo" field to the allowed_fields, the max length of this file_fields is 1.
                let _path = file_fields[0].path.clone(); // 取得暫存檔路徑
                _path
            }, 
            None => {
                error_message = String::from("Server端無法讀取前端上傳的檔案");
                break;
            }
        };
        let mut f = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                error_message = format!("Server端無法開啟前端上傳的檔案, 錯誤: {}", e);
                break;
            }
        };
        let mut buffer = Vec::new();
        match f.read_to_end(&mut buffer) {
            Ok(_) => {},
            Err(e) => {
                error_message = format!("Server端無法讀取前端上傳的檔案({}), 錯誤: {}",report_file, e); 
                break;
            }
        };
        // 寫入資料庫
        match conn.exec_drop("INSERT INTO reports(emp_no, scan_time, scan_file, result, report_file, report_data) VALUES (?, ?, ?, ?, ?, ?)", (user, scan_time, scan_file_name, result, report_file, buffer)).await {
            Ok(_) => {},
            Err(e) => {
                error_message = format!("Server端寫入DB失敗, 錯誤: {}", e);
                break;
            }
        };
        break;
    };

    if !error_message.is_empty() {
        return error_message;
    }

    String::from("ok")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_report_list, get_report_list_post, get_report, upload_report])
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