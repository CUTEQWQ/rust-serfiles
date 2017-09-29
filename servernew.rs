//extern crate time;
extern crate package_handler;
extern crate stream_handler;

//use time::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::sync::{Arc,Mutex};
use std::fs::File;
use std::{thread,time};
use std::io::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use package_handler::*;
use stream_handler::*;
use std::collections::VecDeque;
use std::cmp::PartialEq;


fn main() {
    //using cache
    let listener=TcpListener::bind("0.0.0.0:6060").unwrap();
    let mut f=File::open("test.txt").unwrap();
    let mut vec=Vec::new();
    let file_size=f.read_to_end(&mut vec).unwrap();
    let mut contents = Arc::new(vec);
    //println!("{:?}",contents);
    println!("file read finish! total {} byte from file.",file_size);

    for stream in listener.incoming(){
        println!("in stream");

        let contents = contents.clone();
        let streamthread = thread::spawn(move || {
            let file_size = file_size.clone() as u32;
            let newstream = mystream::new(stream.unwrap());

            //let mut contents = Arc::new(vec);

            let request :Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

            //监听读请求
            let request3 = request.clone();
            let newstream3 = newstream.clone();
            let readth = thread::spawn(move || {
                println!("in read");
                //println!("1:in read,request3:{:?},stream3:{:?}",request3,newstream3.astream);
                loop {
                    if !request3.lock().unwrap().is_empty() {
                        continue;
                    }
                    {
                       // println!("2:in read,request3:{:?},stream3:{:?}",request3,newstream3.astream);
                        let b = newstream3.astream.lock().unwrap();
                        let req_now = head_parser(&b);
                        if req_now.len() == 0 {
                            continue;
                        }
                        println!("Now, get a request:{}", req_now);
                        request3.lock().unwrap().push_back(req_now);
                        println!("request3:{:?}",request3);
                      //  println!("3:in read,request3:{:?},stream3:{:?}",request3,newstream3.astream);
                    }
                    //println!("4:in read,request3:{:?},stream3:{:?}",request3,newstream3.astream);
                    //thread::sleep(time::Duration::from_millis(10));
                }
            });
            println!("request: {:?}", request);

            //写线程
            let request2 = request.clone();
            let newstream2 = newstream.clone();
            let writeth = thread::spawn(move || {
                //println!("1:in write,request2:{:?},stream2:{:?}",request2,newstream2.astream);

                loop {
                    //  println!("in write loop");
                    let mut r = request2.lock().unwrap();
                    let contents2 = contents.clone();

                    while !r.is_empty() {
                        let req = r.pop_front().unwrap();
                        //let req = String::from("download");
                        println!("processing the request: {}", req);
                        if req.to_lowercase().starts_with("download") {
                            let mut package = create_package_message(file_size, &contents2);
                            let size = newstream2.astream.lock().unwrap().write(&package).unwrap();
                            newstream2.astream.lock().unwrap().flush().unwrap();
                            println!("finish request of download, totally {} bytes", size);
                        }
                    }
                    //println!("3:in write,request2:{:?},stream2:{:?}",request2,newstream2.astream);
                }
            });
        });
        println!("over");
    }

    loop{}

}