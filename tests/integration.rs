use ruxnasm::assemble;

#[derive(PartialEq, Eq)]
struct HexDump<'a>(&'a [u8]);

impl<'a> HexDump<'a> {
    pub fn new(binary: &'a [u8]) -> Self {
        Self(binary)
    }
}

impl<'a> std::fmt::Debug for HexDump<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_dump = pretty_hex::pretty_hex(&self.0);
        write!(f, "{}", hex_dump)
    }
}

generator::generate_tests!();
