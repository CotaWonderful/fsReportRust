const express = require('express');
const path = require('path');
const bodyParser = require('body-parser');
const fs = require('fs');
const mysql = require('mysql');
const formidable = require('formidable');

// Initialize
var app = express();
var urlencodedParser = bodyParser.urlencoded({ extended: false });  // 解析text body
var conn_pool = mysql.createPool({
	connectionLimit: 10,
	host: 'localhost',
	user: 'wonderful',
	password: '123456',
	database: 'fscli'
});

// Set defualt header
app.use(function(req, res, next) {
  res.setHeader('X-Frame-Options', 'DENY');
  next();
});

console.log("====== Program Start ======");

function do_mysql_query(sql)
{
	return new Promise((resolve, reject) => {
		conn_pool.getConnection((err, connection) => {
	  	if(err)
				reject(err); // not connected!
			else
			{
	 			 // Use the connection
				connection.query(sql, (error, results, fields) => {
	    		// When done with the connection, release it.
	    		connection.release();
	    		// Handle error after the release.
	    		if(error)
						reject(error);
					else
						resolve(results);
	    		// Don't use the connection here, it has been returned to the pool
				});
			}
		});
	});
}

function render(req, filename, callback) {
    // 加入reverse proxy帶過來的路徑
    let file = __dirname + "/" + filename;
    
    // 檢查header有沒有 "X-nginx-url" ，有的話將他的值嵌入url
	let prefix = req.get("X-nginx-url") ? req.get("X-nginx-url") : "";
	if (prefix) {
		prefix = prefix.replace(/\/$/g, "");
		if (prefix.charAt(0) != "/")
			prefix = "/" + prefix;
	}

    let params = { reverseProxyPrefix : prefix };
    fs.readFile(file, 'utf8', function (err, data) {
        if (err) return callback(err);
        for (var key in params) {
            data = data.replace( RegExp("{" + key + "}", "g") , params[key]);
        }
        callback(null, data); // 用 callback 傳回結果
    });
}

app.use(express.static(__dirname));

// 處理fscliApp丟過來的F-Secure 報告(.html檔)
app.post('/uploadReport' , urlencodedParser, async (req, res)=> {
    const form = formidable({ multiples: true });

    // Parsing form data
    form.parse(req, (err, fields, files) => {
        if (err) {  // Unexpected POST data 
            const errMsg = "Error when parsing post data: " + err.toString();
            console.log(errMsg);
            res.send(errMsg);
            return;
        }
        try {
            // 取得前端上傳的檔案
            let tmpFile = files["scan_file"].filepath;
            let reportFileName = fields["report_file"]; // 報告檔名: xxx.html
            let targetFile = __dirname + "\\" + reportFileName;
            
            // 讀取檔案內容,並寫入db
            fs.readFile(tmpFile, {}, async (err, data)=>{
                let sql = "INSERT INTO reports(emp_no, scan_time, scan_file, result, report_file, report_data) " +
                          "VALUES(?, ?, ?, ?, ?, ?);";
                let content = [fields["user"],      // 員編
                               fields["scan_time"], // 掃描時間
                               fields["scan_file"], // 原檔名
                               fields["result"],    // 掃描結果(integer)
                               fields["report_file"], // 報告檔(.html)
                                data];
                sql = mysql.format(sql, content);
                try {
                    await do_mysql_query(sql);
                }
                catch(e) {
                    res.send("將檔案寫入db時發生錯誤: " + e.toString());
                    return;
                }
                
			    res.send("ok");
            });
            
        }
        catch(ex) {
            const errMsg = "Exception when parsing form data: " + ex.toString();
            console.log(errMsg);
            res.send(errMsg);
        }
    });
});

// 回傳報告檔清單: GET
app.get('/getReportList' , urlencodedParser, async (req, res)=> {
    render(req, "index.html", function(err, data){
		if (err) {
			res.send(err.toString());
			console.log(err.toString());
		}
		else
			res.send(data);
	});
});

// 回傳報告檔清單: POST
app.post('/getReportList' , urlencodedParser, async (req, res)=> {
    try {
        let emp_no = req.body.emp_no;
        let sDate = req.body.sDate + " 00:00:00";
        let eDate = req.body.eDate + " 23:59:59";
        let sql = "SELECT id, scan_time, scan_file FROM reports where emp_no='" + emp_no +
                  "' and scan_time between '" + sDate + 
                  "' and '" + eDate + "';";
        var results = await do_mysql_query(sql);
        res.send(results);
    }
    catch(e) {
        return res.status(500).send("Error whe get file data from db: " + e.toString());
    }
});

// 回傳報告
app.post('/getReport' , urlencodedParser, async (req, res)=> {
    try {
        let id = req.body.id;
        let sql = "SELECT report_file, report_data FROM reports where id='" + id + "'";
        var results = await do_mysql_query(sql);

        // Binary data to string
        var dataStr = Buffer.from(results[0]["report_data"], 'binary').toString();
        res.send(dataStr);
    }
    catch(e) {
        return res.status(500).send("Error whe get file data from db: " + e.toString());
    }
});

const server = app.listen(8089, "127.0.0.1", async ()=>{
    const host = server.address().address;
    const port = server.address().port;
    console.log("fsReportSvr listening at http://%s:%s", host, port);
})
