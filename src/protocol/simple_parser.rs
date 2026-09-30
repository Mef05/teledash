use super::{Parser, TelemtryPacket};

// 2 (magic) + 1 (len) + 1 (sensor_id) + 1 (checksum) = 5 min length
const MIN_PAYLOAD_LENTH: usize = 5;

pub struct SimpleParser {
    buffer: Vec<u8>,
}

impl SimpleParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn calculate_checksum(data: &[u8]) -> u8 {
        data.iter().fold(0, |acc, &x| acc ^ x)
    }
}

impl Parser for SimpleParser {
    fn feed_bytes(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    fn try_parse(&mut self) -> Option<TelemtryPacket> {
        // find magic bytes
        let magic_idx = self.buffer.windows(2).position(|w| w == [0xAA, 0xBB]);

        match magic_idx {
            None => drop(
                self.buffer
                    .drain(..self.buffer.len().saturating_sub_signed(1)),
            ),
            Some(idx) => drop(self.buffer.drain(..idx)),
        };

        if self.buffer.len() < MIN_PAYLOAD_LENTH {
            return None;
        }

        let payload_len = self.buffer[2] as usize;
        let expected_total_size = MIN_PAYLOAD_LENTH + payload_len;

        //wait to get whole packet
        if self.buffer.len() < expected_total_size {
            return None;
        }

        let checksum = self.buffer[expected_total_size - 1];
        if checksum == Self::calculate_checksum(&self.buffer[..expected_total_size - 1]) {
            let packet = TelemtryPacket {
                sensor_id: self.buffer[3],
                payload: self.buffer[4..4 + payload_len].to_vec(),
            };

            self.buffer.drain(..expected_total_size);
            Some(packet)
        } else {
            self.buffer.drain(..2);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_complete_packet() {
        let mut parser = SimpleParser::new();
        
        // Create packet: Magic(2), Len(2), ID(1), Payload(10, 20)
        let mut data = vec![0xAA, 0xBB, 2, 1, 10, 20];
        let checksum = SimpleParser::calculate_checksum(&data);
        data.push(checksum);

        parser.feed_bytes(&data);
        let packet = parser.try_parse();

        // Check if parser successfully returned the packet
        assert!(packet.is_some());
        let p = packet.unwrap();
        assert_eq!(p.sensor_id, 1);
        assert_eq!(p.payload, vec![10, 20]);
    }

    #[test]
    fn test_fragmented_packet() {
        let mut parser = SimpleParser::new();
        
        let mut data = vec![0xAA, 0xBB, 2, 1, 10, 20];
        let checksum = SimpleParser::calculate_checksum(&data);
        data.push(checksum);

        // Feed only the first 3 bytes (incomplete packet)
        parser.feed_bytes(&data[0..3]);
        assert!(parser.try_parse().is_none());

        // Feed the rest of the bytes
        parser.feed_bytes(&data[3..]);
        assert!(parser.try_parse().is_some());
    }

    #[test]
    fn test_multiple_packets_in_buffer() {
        let mut parser = SimpleParser::new();
        
        // Create Packet 1
        let mut p1 = vec![0xAA, 0xBB, 1, 100, 50];
        p1.push(SimpleParser::calculate_checksum(&p1));
        
        // Create Packet 2
        let mut p2 = vec![0xAA, 0xBB, 1, 101, 60];
        p2.push(SimpleParser::calculate_checksum(&p2));

        // Feed both packets at the same time
        parser.feed_bytes(&p1);
        parser.feed_bytes(&p2);

        // The first try_parse should return Packet 1
        let res1 = parser.try_parse();
        assert!(res1.is_some());
        assert_eq!(res1.unwrap().sensor_id, 100);

        // The second try_parse should return Packet 2 (this proves drain() works)
        let res2 = parser.try_parse();
        assert!(res2.is_some());
        assert_eq!(res2.unwrap().sensor_id, 101);
    }
}
