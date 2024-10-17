use md5::{Digest, Md5};
use std::{env::args, fs::File, io::BufReader, time::Instant};
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

    println!("Parte programma");
    cycle(walk_dir);

    // Fine benchmark
    println!(
        "Il calcolo dei checksum ha impiegato {} millisecondi.",
        now.elapsed().as_millis()
    );
}

fn cycle(walk_dir: WalkDir) {
    let mut successi: u32 = 0;
    let mut cartella: u32 = 0;
    let mut errori: u32 = 0;
    for file in walk_dir {
        if file.is_ok() {
            let mut hasher = Md5::new();
            let file_direntry = file.unwrap();

            if file_direntry.path().exists() {
                let file_open = File::open(file_direntry.path());
                if file_direntry.path().is_file() && file_open.is_ok() {
                    let reader = BufReader::new(file_open.unwrap());

                    hasher.update(reader.buffer());
                    let hash = hasher.finalize();

                    println!("{:?} {:?}", hash, file_direntry.path());
                    successi += 1;
                } else if file_direntry.path().is_dir() {
                    println!("{:?} è una cartella", file_direntry.path());
                    cartella += 1;
                } else {
                    println!("Errore");
                    errori += 1;
                }
            } else {
                println!("Non esiste");
                errori += 1;
            }
        }
    }

    println!(
        "Successi {}, Cartelle {}, Errori {}",
        successi, cartella, errori
    );
}
