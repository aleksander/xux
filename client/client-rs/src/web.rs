use state::State;
use std::str;

pub fn responce (buf: &[u8], state: &State) -> String {
    let buf = str::from_utf8(buf).unwrap();
    println!("render: {:?}", buf);
    if buf.starts_with("GET /") {
        let pattern: &[_] = &['\r','\n'];
        let crlf = buf.find(pattern).unwrap_or(buf.len());
        _responce(state, &buf[5..crlf]).unwrap_or("HTTP/1.1 404 Not Found\r\n\r\n".to_string())
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    }
}

fn _responce (state: &State, buf: &str) -> Option<String> {
    if buf.starts_with(" ") {
        let body = "<html> \r\n\
                        <head> \r\n\
                            <title></title> \r\n\
                            <script src=\"http://code.jquery.com/jquery-1.11.3.min.js\" stype=\"text/javascript\"></script> \r\n\
                            <script type=\"text/javascript\"> \r\n\
                                $(document).ready(function(){ \r\n\
                                    $('#getdata-button').on('click', function(){ \r\n\
                                        $.get('http://localhost:33000/data', function(data) { \r\n\
                                            $('#showdata').html(\"<p>\"+data+\"</p>\"); \r\n\
                                        }); \r\n\
                                    }); \r\n\
                                }); \r\n\
                            </script> \r\n\
                        </head> \r\n\
                        <body> \r\n\
                            <a href=\"#\" id=\"getdata-button\">C</a> \r\n\
                            <div id=\"showdata\"></div> \r\n\
                        </body> \r\n\
                    </html>\r\n\r\n";
        Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n", body.len()) + &body)
    } else if buf.starts_with("env ") {
        // {
        //   res:[{id:id,name:name}],
        //   obj:[{},{}],
        //   wid:[{},{}],
        //   map:[z,z,...,z]
        // }

        let mut body = "{\"res\":[".to_string();

        let mut period = "";
        for (id,name) in &state.resources {
            body = body + &format!("\r\n{}{{\"id\":{},\"name\":\"{}\"}}", period, id, name);
            period = ",";
        }

        body = body + "],\"obj\":[";

        period = "";
        for o in state.objects.values() {
            let resname = match state.resources.get(&o.resid) {
                Some(res) => res.as_str(),
                None      => "null"
            };
            body = body + &format!("\r\n{}{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}}", period, o.x, o.y, o.resid, resname);
            period = ",";
        }

        body = body + "],\"wid\":[";

        period = "";
        for (id,w) in &state.widgets {
            body = body + &format!("\r\n{}{{\"id\":{},\"name\":\"{}\",\"parent\":\"{}\"}}", period, id, w.typ, w.parent);
            period = ",";
        }

        body = body + "],\"map\":[";

        period = "";
        match state.hero_grid() {
            Some(/*grid*/_) => {
                for /*y*/_ in 0..100 {
                    for /*x*/_ in 0..100 {
                        body = body + &format!("{}{}", period, 0/*grid.z[x+y*100]*/); //FIXME grids Z buffers moved from State to Client
                        period = ",";
                    }
                }
            }
            //TODO send one Null instead of 10000 zeroes
            None => {
                for _ in 0..100 {
                    for _ in 0..100 {
                        body = body + &format!("{}{}", period, 0);
                        period = ",";
                    }
                }
            }
        }

        body = body + "]}";
        Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
    } else if buf.starts_with("objects ") {
        let mut body = String::new();
        for o in state.objects.values() {
            let resname = match state.resources.get(&o.resid) {
                Some(res) => res.as_str(),
                None      => "null"
            };
            body = body + &format!("{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}},", o.x, o.y, o.resid, resname);
        }
        body = "[ ".to_string() + &body[..body.len()-1] + " ]";
        Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
    } else if buf.starts_with("widgets ") {
        let mut body = String::new();
        for (id,w) in &state.widgets {
            body = body + &format!("{{\"id\":{},\"name\":\"{}\",\"parent\":\"{}\"}},", id, w.typ, w.parent);
        }
        body = "[ ".to_string() + &body[..body.len()-1] + " ]";
        Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
    } else if buf.starts_with("resources ") {
        //TODO
        Some("HTTP/1.1 404 Not Implemented\r\n\r\n".to_string())
    } else if buf.starts_with("go/") {
        //FIXME should NOT be implemented for web. web is for view only
        //println!("GO: {} {}", x, y);
        //if let Err(e) = client.go(x,y) {
        //    println!("ERROR: client.go: {:?}", e);
        //}
        let tmp1: Vec<&str> = buf.split(' ').collect();
        println!("TMP1: {:?}", tmp1);
        let tmp2: Vec<&str> = tmp1[1].split('/').collect();
        println!("TMP2: {:?}", tmp2);
        if tmp2.len() > 3 {
            let /*x*/_: i32 = match str::FromStr::from_str(tmp2[2]) { Ok(v) => v, Err(_) => 0 };
            let /*y*/_: i32 = match str::FromStr::from_str(tmp2[3]) { Ok(v) => v, Err(_) => 0 };
            //self.url = Some(Url::Go(x,y));
        } else {
            //self.url = Some(Url::Go(0,0));
        }
        Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
    } else if buf.starts_with("quit ") {
        /*FIXME if let Err(e) = state.close() {
            println!("ERROR: client.close: {:?}", e);
        }*/
        Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
    } else {
        Some("HTTP/1.1 404 Not Found\r\n\r\n".to_string())
    }
}
