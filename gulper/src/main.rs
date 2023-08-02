use std::{io, result};

type U1k = [u64; 128];
type U1m = [U1k; 1024];
type U1g = [U1m; 1024];


fn make_1k() -> U1k {
    [0u64; 128]
}

fn make_1m() -> U1m {
    [[0u64; 128]; 1024]
}

fn make_1g() -> U1g {
    [[[0u64; 128]; 1024]; 1024]
}

fn main() {
    let mut mbs = Vec::new();
    loop {
        println!("Current megabytes: {}", mbs.len());
        let mut answer = String::new();

        if io::stdin().read_line(&mut answer).is_err() {
            break;
        }

        for _ in 0..128 {
            mbs.push(make_1m());
        }
    }
}
