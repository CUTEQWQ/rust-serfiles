extern crate package_handler;
extern crate bichannels;

use std::net::TcpStream;
use std::collections::VecDeque;
use std::thread;
use std::net::TcpListener;
use std::sync::{Arc,mpsc};
use std::fs::File;
use std::io::prelude::*;
use package_handler::*;


fn main() {
    let listener=TcpListener::bind("0.0.0.0:6061").unwrap();
    let mut f = File::open("test.txt").unwrap();
    let mut vec = Vec::new();
    let file_size = f.read_to_end(&mut vec).unwrap() as u32;
    let mut contents = Arc::new(vec);
    println!("file read finish! total {} bytes.",file_size);

    for stream in listener.incoming() {
        let contents = contents.clone();
        let mut stream = stream.unwrap();

        let athread = thread::spawn(move||{

            let bichannels::BiChannel{e1:e1, e2:e2} = bichannels::BiChannel::new();
            let mut requests :VecDeque<String> = VecDeque::new();
            let t = thread::spawn(move|| {

                let request_now = head_parser(&stream);
                if !request_now.is_empty() {
                    println!("get a new request: {}", request_now);
                    requests.push_back(request_now);
                    e1.send(requests);
                }
                loop {
                    //read the stream
                    if let Ok(mut requests) = e1.try_recv() {
                        let request_now = head_parser(&stream);
                        if !request_now.is_empty() {
                            println!("get a new request: {}", request_now);
                            requests.push_back(request_now);
                            e1.send(requests);
                        }
                    }
                    if let Ok(mut requests) = e2.try_recv() {
                        println!("requests queue: {:?}",requests);
                        if !requests.is_empty() {
                            let request = requests.pop_front().unwrap();
                            println!("processing the request:{}",request);
                            if request.to_lowercase().starts_with("download") {
                                let mut package = create_package_message(file_size, &contents);

                                let size = stream.write(&package).unwrap();
                                stream.flush().unwrap();
                                println!("finish request of download, totally {:?} bytes", size);
                                e2.send(requests);
                            }
                                else{
                                    println!("its other type request!");
                                }
                        }
                            else{}
                    }
                }
            });
            t.join().unwrap();
        });
    }
    loop{}
}
