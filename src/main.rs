use md5::{Digest, Md5};
use std::{
    borrow::Borrow,
    env::args,
    fs::{read_to_string, File},
    io::BufReader,
    os::fd::AsRawFd,
    time::Instant,
};
use walkdir::WalkDir;

// https://reveng.sourceforge.io/crc-catalogue/all.htm
// CRC-32/CKSUM

fn main() {
    // args[1] = cartella
    // args[2] = file output
    let args: Vec<String> = args().collect();

    if args.len() != 3 {
        println!("Errore, passa 3 argomenti. cartella file_output");
        return;
    }

    let walk_dir: WalkDir = WalkDir::new(args[1].clone()).follow_links(true);

    // Avvia time benchmark
    let now = Instant::now();

    cycle(walk_dir);

    // Fine benchmark
    let elapsed_time = now.elapsed();
    println!(
        "Il calcolo dei checksum ha impiegato {} secondi.",
        elapsed_time.as_secs()
    );
}

fn cycle(walk_dir: WalkDir) {
    let mut successi: u32 = 0;
    let mut errori: u32 = 0;
    for file in walk_dir {
        if !file.is_err() {
            let mut hasher = Md5::new();
            let file_path = file.unwrap();
            let file_string = read_to_string(file_path.path());
            if !file_string.is_err() {
                hasher.update(file_string.unwrap());
                let hash = hasher.finalize();
                if file_path.path().is_file() && file_path.path().exists() {
                    println!("{:?} {:?}", hash, file_path.path());
                    successi = successi + 1;
                }
            } else {
                // println!("Errore lettura del file o è una cartella -> {:?}", file_path);
                errori = errori + 1;
            }
        } else {
            // println!("Errore, file o cartella non esiste");
            errori = errori + 1;
        }
    }

    println!("Successi {}, Errori {}", successi, errori);

    return;
}
