use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

fn main() {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/ttyS0")
        .unwrap();

    let mut buf = [0; 1];
    loop {
        f.read(&mut buf).unwrap();

        if buf[0] == 0x3 {
            println!("Interrupt received");
            break;
        }

        f.write(&buf[..1]).unwrap();
    }
}
