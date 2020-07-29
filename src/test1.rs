use super::ApplicationError;
use std::sync::mpsc;
use std::thread;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Write;

#[allow(dead_code)]
fn test1() -> Result<(), ApplicationError> {
    let (tx, rx) = mpsc::channel();
    let (col, height) = crossterm::terminal::size().unwrap();
    println!("{}, {}", col, height);
    let in_thread = thread::spawn(move || -> std::io::Result<()> {
        let sin = std::io::stdin();
        let mut bufin = String::new();
        bufin.reserve(4096);
        loop {
            let bytesread = sin.read_line(&mut bufin)?;
            if bytesread > 0 {
                match tx.send(bufin.clone()) {
                    Ok(v) => v,
                    Err(e) => return Err(IoError::new(ErrorKind::Other, format!("{:?}", e))),
                }
                bufin.clear();
            } else {
                break;
            }
        }
        Ok(())
    });
    let out_thread = thread::spawn(move || -> std::io::Result<()> {
        let mut sout = std::io::stdout();
        for recv in rx {
            sout.write(&recv.as_bytes())?;
        }
        Ok(())
    });
    in_thread.join().unwrap().unwrap();
    out_thread.join().unwrap().unwrap();
    Ok(())
}

