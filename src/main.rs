#[macro_use] extern crate rocket;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::fs::FileServer;
use rocket::fs::NamedFile;
use rocket::fs::relative;
use std::path::{Path, PathBuf};
use rocket::form::Form;
use rocket::tokio::time::{sleep, Duration};
use mysql_async::{OptsBuilder, Conn, Opts};
use mysql_async::prelude::*;

#[derive(Serialize, Deserialize)]
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

/*#[post("/json", format = "json", data = "<data>")]
fn post_json(data: Json<Message>) -> Json<Message> {
    //data
}*/

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, message, getReportList, getReport])
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