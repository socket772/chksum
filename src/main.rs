use std::{
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

//
static READY_BUFFER: Mutex<u8> = Mutex::new(0);

// Array degli stati dei buffer
// 0 = empty
// 1 = ready
// 2 = working
// 3 = to delete
static STATUS_VECTOR: RwLock<[u8; MAX_LENGHT]> = RwLock::new([0; MAX_LENGHT]);

static mut BUFFER_VECTOR: Vec<Vec<u8>> = Vec::new();

static mut FILEPATH_VECTOR: Vec<String> = Vec::new();

fn main() {
    // args[1] = cartella
    // args[2] = file output
    let args: Vec<String> = args().collect();

    if args.len() != 3 {
        println!("Errore, passa 3 argomenti. cartella file_output");
        return;
    }

    for i in 0..MAX_LENGHT {
        unsafe {
            BUFFER_VECTOR[i] = Vec::new();
            FILEPATH_VECTOR[i] = String::new();
        }
    }

    // Crea file di output
    let mut output_file: File = File::create(args[2].clone()).unwrap();
    output_file.write_all(b"").unwrap();
    writeln!(output_file, "cksum,size,file").unwrap();

    println!("Parte programma");
    // Avvia time benchmark
    let now = Instant::now();
    let walk_dir: WalkDir = WalkDir::new(args[1].clone()).follow_links(true);
    ciclo_lettore(walk_dir);
    ciclo_checksum(&output_file);

    // Fine benchmark
    println!(
        "Il calcolo dei checksum ha impiegato {} millisecondi.",
        now.elapsed().as_millis()
    );
}

// da aggiornare
fn ciclo_checksum(mut output_file: &File) {
    let mut successi: u32 = 0;
    let mut cartella: u32 = 0;
    let mut errori: u32 = 0;

    // prendo il lock dei buffer pronti
    let mut lock_ready_buffer = READY_BUFFER.lock().unwrap();

    if (*lock_ready_buffer) > 0 {
        *lock_ready_buffer = *lock_ready_buffer - 1;
        drop(lock_ready_buffer);
        for i in 0..MAX_LENGHT {
            // prendo il lock in scrittura per annunciare (se libero lo slot) l'uso del buffer in posizione i
            let lock_status_write = STATUS_VECTOR.write().unwrap();
            // se è pronto allora inizia l'elaborazione
            if (*lock_status_write)[i] == 1 {
                // indica che il buffer p in uso
                (*lock_status_write)[i] = 2;
                // lascia il lock
                drop(lock_status_write);
                // calcola il checksum
                unsafe {
                    let checksum = CHECKSUMMER.checksum(&BUFFER_VECTOR[i]);
                    println!("{:?},{}", checksum, FILEPATH_VECTOR[i]);
                }
            }
        }
    } else {
        todo!();
    }

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
    for file in walk_dir {
        if file.is_ok() {
            let file_direntry = file.unwrap();
            for i in 0..MAX_LENGHT {
                let lock_read = STATUS_VECTOR.read().unwrap();
                if ((*lock_read)[i] == 0 || (*lock_read)[i] == 3) && file_direntry.path().exists() {
                    let file_open = File::open(file_direntry.path());

                    if file_direntry.path().is_file() && file_open.is_ok() {
                        let buffer: &mut Vec<u8> = &mut Vec::<u8>::new();
                        let _ = file_open.unwrap().read_to_end(buffer);

                        unsafe {
                            BUFFER_VECTOR[i] = buffer.clone();
                            FILEPATH_VECTOR[i] = file_direntry.path().to_str().unwrap().to_string();
                        }
                        drop(lock_read);
                        let mut lock_write = STATUS_VECTOR.write().unwrap();
                        (*lock_write)[i] = 1;
                        drop(lock_write);
                        let mut lock_ready = READY_BUFFER.lock().unwrap();
                        *lock_ready = *lock_ready + 1;
                        drop(lock_ready);
                    }
                }
            }
        }
    }
}
