use std::{
    env::args,
    fs::File,
    io::{Read, Write},
    sync::{Mutex, RwLock},
    thread,
    time::Instant,
};
use walkdir::{DirEntry, WalkDir};

// https://reveng.sourceforge.io/crc-catalogue/all.htm
// CRC-32/CKSUM

// Formato
// crc,size(B),filepath
const CHECKSUMMER: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

// Successi, Cartelle, Errori
static STATS: Mutex<[u32; 3]> = Mutex::new([0, 0, 0]);

// Dimensione massima array
const MAX_LENGHT: usize = 8;

// Buffer contenente il numero di buffer pronti all'uso
static READY_BUFFER: Mutex<usize> = Mutex::new(0);

// Array degli stati dei buffer
// 0 = empty
// 1 = ready
// 2 = working
// 3 = to delete
static STATUS_VECTOR: RwLock<[u8; MAX_LENGHT]> = RwLock::new([0; MAX_LENGHT]);

// vettore di buffer
static mut BUFFER_VECTOR: Vec<Vec<u8>> = Vec::new();

// vettore che contiene i percorsi dei file nel vettore dei buffer
static mut FILEPATH_VECTOR: Vec<String> = Vec::new();

// booleano che indica la fine del programma, fa chiudere tutti i thread
static BOOL_END: RwLock<bool> = RwLock::new(false);

// booleano che indica se il thread che controlla il buffer è stato creato
// true = controller da creare
// false = controller creato
static BOOL_TYPE: Mutex<bool> = Mutex::new(true);

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
            BUFFER_VECTOR.push(Vec::new());
            FILEPATH_VECTOR.push(String::new());
        }
        (*STATUS_VECTOR.write().unwrap())[i] = 0;
    }

    println!("Parte programma");
    // Avvia benchmark
    let now = Instant::now();
    // Crea file di output
    // let mut output_file: File = File::create(args[2].clone()).unwrap();
    // output_file.write_all(b"").unwrap();
    // writeln!(output_file, "cksum,size,file").unwrap();

    let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];
    for i in 0..9 {
        // Qui metto in nuovo thread nell'array, è come una lista.
        thread_vector.push(thread::spawn(move || {
            let mut lock_tipo = BOOL_TYPE.lock().unwrap();

            // verifico se esiste almeno un buffer
            if *lock_tipo {
                println!("Creo thread lettore");
                *lock_tipo = false;
                drop(lock_tipo);
                ciclo_lettore(i);
            } else {
                println!("Creo thread checksum");
                ciclo_checksum(i);
            }
        }));
    }

    for thread_element in thread_vector {
        let _ = thread_element.join();
    }

    // Fine benchmark
    println!(
        "Il calcolo dei checksum ha impiegato {} millisecondi.",
        now.elapsed().as_millis()
    );
}

// da aggiornare
fn ciclo_checksum(id: usize) {
    let mut successi: u32 = 0;
    // let mut cartella: u32 = 0;
    // let mut errori: u32 = 0;

    loop {
        // Controlla se è stata chiamata la fine del programma
        if *BOOL_END.read().unwrap() {
            println!("{} - CHKSUM: terminato", id);
            break;
        }

        // prendo il lock dei buffer pronti
        let lock_ready_buffer = READY_BUFFER.lock().unwrap();

        // se ci sono buffer pronti, allora elabora il checksum
        if (*lock_ready_buffer) > 0 {
            drop(lock_ready_buffer);
            println!(
                "{} - CHKSUM: almeno 1 buffer pronto, inizio elaborazione",
                id
            );
            for i in 0..MAX_LENGHT {
                // prendo il lock in scrittura per annunciare (se libero lo slot) l'uso del buffer in posizione i
                let mut lock_status_write = STATUS_VECTOR.write().unwrap();
                // se è pronto allora inizia l'elaborazione
                if (*lock_status_write)[i] == 1 {
                    // println!("CHKSUM: Trovato il buffer libero");
                    // indica che il buffer i in uso
                    (*lock_status_write)[i] = 2;
                    // lascia il lock
                    drop(lock_status_write);
                    // calcola il checksum
                    unsafe {
                        let checksum = CHECKSUMMER.checksum(&BUFFER_VECTOR[i]);
                        // stampa il checksum, poi diventerà una scrittura su file
                        println!("{} - {:?},{}", id, checksum, FILEPATH_VECTOR[i]);
                    }
                    successi += 1;
                    (*STATUS_VECTOR.write().unwrap())[i] = 3;
                    // prendo il lock dei buffer pronti per indicare che si è liberato un buffer
                    let mut lock_ready_buffer = READY_BUFFER.lock().unwrap();
                    *lock_ready_buffer -= 1;
                }
            }
        } else {
            println!("{} - CHKSUM: non ho trovato buffer pronti", id)
        }
    }
    // println!("Successi Locali {}", successi);
    // Aggiungi i successi all'array globale
    let mut lock_stats = STATS.lock().unwrap();
    (*lock_stats)[0] += successi;
}

fn ciclo_lettore(id: usize) {
    // Trovo tutti i file/cartelle
    let args: Vec<String> = args().collect();
    let walk_dir: WalkDir = WalkDir::new(args[1].clone()).follow_links(true);

    for file in walk_dir.into_iter().flatten() {
        // Controllo se posso accedere alla cartella
        if file.path().exists() && file.path().is_file() {
            loader(file);
        }
    }
    println!("{} - BUFF: Segnale di chiusura", id);
    // mando il segnale di fine dei buffer
    let mut lock_end = BOOL_END.write().unwrap();
    *lock_end = true;
}

// Funzione che effettua il caricamento in ram effettivo
fn loader(file: DirEntry) {
    println!("{:?}", file);
    let mut caricato = false;
    while !caricato {
        // controlla se il buffer è pieno
        let lock_ready = READY_BUFFER.lock().unwrap();
        // println!("BUFF: buffer riempito {}/{}", *lock_ready, MAX_LENGHT);
        if *lock_ready == MAX_LENGHT {
            println!("BUFF: buffer pieno -> {:?}", file.path());
            drop(lock_ready);
            continue;
        }
        drop(lock_ready);
        for i in 0..MAX_LENGHT {
            let lock_read = STATUS_VECTOR.read().unwrap();

            // controllo se il buffer è usabile e se il percorso esiste davvero
            // println!("BUFF: indice,stato -> {},{}", i, (*lock_read)[i]);
            if (*lock_read)[i] == 0 || (*lock_read)[i] == 3 {
                // println!("BUFF: buffer vuoto trovato -> {:?}", file.path());
                let file_open = File::open(file.path());

                // Verifico che è un file e che l'apertura è andata bene
                if file_open.is_ok() {
                    // println!("BUFF: file aperto -> {:?}", file.path());

                    // Carico il file in ram
                    let buffer: &mut Vec<u8> = &mut Vec::<u8>::new();
                    let _ = file_open.unwrap().read_to_end(buffer);

                    // Salvo nel buffer il file
                    unsafe {
                        BUFFER_VECTOR[i] = buffer.clone();
                        FILEPATH_VECTOR[i] = file.path().to_str().unwrap().to_string();
                    }
                    drop(lock_read);
                    println!("BUFF: caricato con successo -> {:?}", file);

                    // cambio lo stato del buffer nello slot i
                    let mut lock_write = STATUS_VECTOR.write().unwrap();
                    (*lock_write)[i] = 1;
                    drop(lock_write);
                    let mut lock_ready_buffer = READY_BUFFER.lock().unwrap();

                    // aumento di 1 l'indicatore di buffer pronti
                    *lock_ready_buffer += 1;
                    drop(lock_ready_buffer);

                    // indico che il file è stato caricato e interrompo il ciclo
                    caricato = true;
                    break;
                }
            }
        }
    }
}
