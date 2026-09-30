pub mod tcpuart;

use crate::error::TransportError;
use mio::{Registry, Token};

const BUF_SIZE: usize = 0x1000;

// main interface
pub trait DataSource {
    //adds source to eventloop
    fn register(&mut self, registry: &Registry, token: Token) -> Result<(), TransportError>;

    fn connect(&mut self) -> Result<(), TransportError>;

    fn read_bytes(&mut self) -> Result<Option<Vec<u8>>, TransportError>;

    fn write_bytes(&mut self, data: &[u8]) -> Result<usize, TransportError>;
}
