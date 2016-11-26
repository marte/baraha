extern crate baraha;

use std::thread;

use baraha::{server, client};

#[test]
fn host_and_play() {
    let mut joins = vec![];
    for _ in 0..4 {
        joins.push(thread::spawn(|| client::bot("localhost".into())));
    }
    server::host();
    for join in joins {
        join.join().unwrap();
    }
}
