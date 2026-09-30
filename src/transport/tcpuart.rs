use super::{DataSource, BUF_SIZE};
use crate::error::TransportError;
use mio::net::TcpStream;
use mio::{Interest, Registry, Token};
use std::error::Error;
use std::io::{self, ErrorKind, Read, Write};
use std::net::SocketAddr;

pub struct TcpUartSource {
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl TcpUartSource {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr, stream: None }
    }
}

impl DataSource for TcpUartSource {
    fn register(&mut self, registry: &Registry, token: Token) -> Result<(), TransportError> {
        if let Some(stream) = &mut self.stream {
            registry.register(stream, token, Interest::READABLE)?;
            Ok(())
        } else {
            Err(TransportError::Disconnected)
        }
    }

    fn connect(&mut self) -> Result<(), TransportError> {
        self.stream = Some(TcpStream::connect(self.addr)?);
        Ok(())
    }

    fn read_bytes(&mut self) -> Result<Option<Vec<u8>>, TransportError> {
        if let Some(stream) = &mut self.stream {
            let mut buf = [0u8; BUF_SIZE];
            match stream.read(&mut buf) {
                Ok(0) => Err(TransportError::Disconnected),
                Ok(n) => Ok(Some(buf[..n].to_vec())),
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        Ok(None)
                    } else {
                        Err(TransportError::Io(e))
                    }
                }
            }
        } else {
            Err(TransportError::Disconnected)
        }
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        if let Some(stream) = &mut self.stream {
            match stream.write(data) {
                Ok(n) => Ok(n),
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        Ok(0)
                    } else {
                        Err(TransportError::Io(e))
                    }
                }
            }
        } else {
            Err(TransportError::Disconnected)
        }
    }
}
