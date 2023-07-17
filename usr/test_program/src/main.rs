use std::time::Duration;

fn main() {
    #[derive(Debug)]
    struct A {
        a: u32,
        b: i32,
    }
    let a = A { a: 1234, b: -31 };
    for _ in 0..10 {
        std::thread::sleep(Duration::from_secs(1));
        println!("Hello {:?}", a);
    }
}
