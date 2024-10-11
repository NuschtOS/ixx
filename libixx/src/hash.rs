use crc::{Crc, CRC_8_LTE};

const CRC: Crc<u8> = Crc::<u8>::new(&CRC_8_LTE);

pub fn hash(option: &str) -> u8 {
  CRC.checksum(option.as_bytes())
}
