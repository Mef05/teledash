mod simple_parser;

#[derive(Debug, PartialEq)]
pub struct TelemtryPacket {
    pub sensor_id: u8,
    pub payload: Vec<u8>,
}

pub trait Parser {
    fn feed_bytes(&mut self, data: &[u8]);

    fn try_parse(&mut self) -> Option<TelemtryPacket>;
}
