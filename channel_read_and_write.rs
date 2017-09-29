extern crate package_handler;
extern crate stream_handler;
extern crate bichannels;

use std::net::TcpStream;
use std::collections::VecDeque;
use std::thread;
use std::net::TcpListener;
use std::sync::{Arc,mpsc};
use std::fs::File;
use std::io::prelude::*;
use package_handler::*;
use stream_handler::*;
use bichannels::BiChannel;


fn main() {
    let listener=TcpListener::bind("0.0.0.0:6061").unwrap();
    let mut f = File::open("test.pdf").unwrap();
    let mut vec = Vec::new();
    let file_size = f.read_to_end(&mut vec).unwrap() as u32;
    let mut contents = Arc::new(vec);
    println!("file read finish! total {} bytes.",file_size);

    for stream in listener.incoming() {
        let contents = contents.clone();
        let mut stream = stream.unwrap();

        let bichannels::BiChannel{e1:e1, e2:e2} = bichannels::BiChannel::new();

        let mut requests :VecDeque<String> = VecDeque::new();

        let t = thread::spawn(move|| {

            //first need to block for request
            let request_first = head_parser_blocking(&stream, false);
            println!("get a new request: {}",request_first);
            requests.push_back(request_first);

            let writebuf = create_package_message(file_size.clone(),&contents);

            //read and write

            //read request
            //if block,starting write
            //if block,starting read
            loop {
                let mut channel = bichannels::BiChannel{e1:e1, e2:e2};
                channel.e1.send(requests);
                let (headfinish,request_next_head) = read_from_stream(&stream,4, &writebuf,&channel);
                if headfinish {
                    let len = package_len_small(request_next_head);
                    let (bodyfinish,request_next) = read_from_stream(&stream, len, &writebuf,&channel);
                    if bodyfinish{
                        requests.push_back(request_next);
                        e1.send(requests);
                        write_to_stream(&stream,&writebuf,channel,len);
                    }
                }
                else { write_to_stream(&stream,&writebuf,channel,len); }
            }

        });
        t.join().unwrap();
    }
    loop{}
}
