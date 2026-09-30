mod error;
mod protocol;
mod transport;

use mio::{Events, Poll, Token};
use std::time::Duration;
use transport::tcpuart::TcpUartSource;
use transport::DataSource;

const SOURCE_TOKEN: Token = Token(0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inițializăm Event Loop-ul (Poll) și un buffer pentru evenimente
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    // 2. Definim adresa de test (ne vom conecta la un server local)
    let addr = "127.0.0.1:15113".parse()?;
    let mut source = TcpUartSource::new(addr);

    // 3. Conectarea la sursă
    println!("Se conectează la serverul {}...", addr);
    source.connect()?;
    println!("Conectat cu succes!");

    // 4. Înregistrăm socket-ul în poll (Event Loop)
    source.register(poll.registry(), SOURCE_TOKEN)?;

    println!("Event loop pornit. Așteptăm date de la server... (scrie ceva în netcat)");

    // 5. Bucla infinită a Event Loop-ului
    loop {
        // Așteptăm până la 1 secundă după evenimente de la sistemul de operare
        poll.poll(&mut events, Some(Duration::from_secs(1)))?;

        for event in events.iter() {
            if event.token() == SOURCE_TOKEN && event.is_readable() {
                // OS-ul ne-a trezit pentru că socket-ul are date
                match source.read_bytes() {
                    Ok(Some(bytes)) => {
                        // Transformăm array-ul de bytes într-un string (lossy, in caz de non-utf8)
                        let text = String::from_utf8_lossy(&bytes);
                        println!(">> Primit de rețea ({} bytes): {:?}", bytes.len(), text);
                    }
                    Ok(None) => {
                        // A dat WouldBlock (alarma falsă, nu avem date complete momentan)
                        continue;
                    }
                    Err(e) => {
                        println!(">> Conexiune închisă sau eroare fatală: {:?}", e);
                        println!("Ieșire curată.");
                        return Ok(());
                    }
                }
            }
        }
    }
}
