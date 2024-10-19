use std::{
    default,
    env::args,
    fs::File,
    io::{Read, Write},
    sync::{Mutex, RwLock},
    time::Instant,
};
use walkdir::WalkDir;

// https://reveng.sourceforge.io/crc-catalogue/all.htm
// CRC-32/CKSUM

// Formato
// crc,size(B),filepath
const CHECKSUMMER: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

// Successi, Cartelle, Errori
static STATS: Mutex<[i32; 3]> = Mutex::new([0, 0, 0]);

// Dimensione massima array
const MAX_LENGHT: usize = 8;

const DEFAULT_DATASET: DataSet = DataSet {
    status: 0,
    buffer: &mut Vec::new(),
};

// Array dei buffer
// 0 = empty
// 1 = ready
// 2 = working
// 3 = to delete

static BUFFER_ARRAY: [Mutex<DataSet>; MAX_LENGHT] = [
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
    Mutex::new(DEFAULT_DATASET),
];

struct DataSet<'a> {
    status: usize,
    buffer: &'a mut Vec<u8>,
}

fn main() {
    // args[1] = cartella
    // args[2] = file output
    let args: Vec<String> = args().collect();

    if args.len() != 3 {
        println!("Errore, passa 3 argomenti. cartella file_output");
        return;
    }

    println!("Parte programma");
    // Avvia time benchmark
    let now = Instant::now();
    let walk_dir: WalkDir = WalkDir::new(args[1].clone()).follow_links(true);
    ciclo_checksum(walk_dir, args);

    // Fine benchmark
    println!(
        "Il calcolo dei checksum ha impiegato {} millisecondi.",
        now.elapsed().as_millis()
    );
}

// da aggiornare
fn ciclo_checksum(walk_dir: WalkDir, args: Vec<String>) {
    let mut successi: u32 = 0;
    let mut cartella: u32 = 0;
    let mut errori: u32 = 0;
    let mut output_file = File::create(args[2].clone()).unwrap();
    output_file.write_all(b"").unwrap();
    writeln!(output_file, "cksum,size,file").unwrap();
    for file in walk_dir {
        if file.is_ok() {
            let file_direntry = file.unwrap();

            if file_direntry.path().exists() {
                let file_open = File::open(file_direntry.path());

                if file_direntry.path().is_file() && file_open.is_ok() {
                    let buffer: &mut Vec<u8> = &mut vec![];
                    let buffer_result = file_open.unwrap().read_to_end(buffer);

                    if buffer_result.is_ok() {
                        writeln!(
                            output_file,
                            "{:?},{:?},{}",
                            CHECKSUMMER.checksum(buffer),
                            buffer.len(),
                            file_direntry.path().to_str().unwrap()
                        )
                        .unwrap();

                        successi += 1;
                    }
                } else if file_direntry.path().is_dir() {
                    // println!("{:?} è una cartella", file_direntry.path());
                    cartella += 1;
                } else {
                    println!("Errore");
                    errori += 1;
                }
            } else {
                println!("Non esiste");
                errori += 1;
            }
        } else {
            println!("Errore");
            errori += 1;
        }
    }

    println!(
        "Successi {}, Cartelle {}, Errori {}",
        successi, cartella, errori
    );
}

fn ciclo_lettore(walk_dir: WalkDir) {
    let mut data_result;
    for file in walk_dir {
        if file.is_ok() {
            let file_direntry = file.unwrap();
            for element in 0..7 {
                data_result = BUFFER_ARRAY[element].lock().unwrap();
                if *&data_result.status == 0 {
                    if file_direntry.path().exists() {
                        let file_open = File::open(file_direntry.path());

                        if file_direntry.path().is_file() && file_open.is_ok() {
                            let buffer: &mut Vec<u8> = &mut vec![];
                            let buffer_result = file_open.unwrap().read_to_end(buffer);
                            if buffer_result.is_ok() {
                                *data_result.buffer = buffer;
                            }
                        }
                    }
                }
            }
        }
    }
}
