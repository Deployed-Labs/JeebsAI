use std::io;

pub fn encode_all(input: &[u8], level: u32) -> io::Result<Vec<u8>> {
    // 0 is default level in zstd, mapping 1-21 roughly to zstd levels
    zstd::stream::encode_all(input, level as i32)
}

pub fn decode_all(input: &[u8]) -> io::Result<Vec<u8>> {
    zstd::stream::decode_all(input)
}