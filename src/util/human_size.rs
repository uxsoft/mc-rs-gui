use bytesize::ByteSize;

pub fn format_size(bytes: u64) -> String {
    ByteSize(bytes).to_string()
}
