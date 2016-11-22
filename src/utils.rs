use std::io::prelude::*;
use std::net::TcpStream;

pub fn read_line(mut stream: &TcpStream) -> String {
    let mut line = String::with_capacity(256);
    loop {
        let mut buf = [0u8];
        let size = stream.read(&mut buf).expect("read error");
        assert_eq!(1, size); // we don't expect EOF in our protocol
        line.push(buf[0] as char);
        if buf[0] == 10 { // '\n'
            return line
        }
    }
}
