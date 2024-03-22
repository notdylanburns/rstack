#[derive(Debug)]
pub struct Ipv4Header {
    version: u8,        // 4 bits
    ihl: u8,            // 4 bits

    // Type of Service (8 bits)
    precedence: u8,     // 3 bits
    delay: bool,
    throughput: bool,
    reliability: bool,
    reserved_0: u8,     // 2 bits

    total_length: u16,
    identification: u16,

    // Flags (3 bits)
    reserved_1: bool,
    dont_fragment: bool,
    more_fragments: bool,

    fragment_offset: u16,    // 13 bits

    ttl: u8,
    proto: u8,
    checksum: u16,

    source_addr: u32,
    dest_addr: u32,

    options: Vec<u8>,
}

macro_rules! bool_to_bit {
    ($n:literal, $e:expr) => {
        if $e {
            1 << $n
        } else {
            0
        }
    };
}

macro_rules! set_if_some {
    ($v:expr, $e:expr) => {
        if let Some(v) = $e {
            $v = v;
        }
    };
}

impl Ipv4Header {
    fn _checksum(bytes: &[u8]) -> u16 {
        let total = 0u16; 

        for (idx, byte_pair) in bytes.chunks(2).enumerate() {
            Self::_checksum_add_bytes(total, byte_pair[0], byte_pair[1]);
        };

        !total
    }

    fn _checksum_add(acc: u16, v: u16) -> u16 {
        let (mut total, overflow) = acc.overflowing_add(v);
        if overflow {
            total += 1;
        }

        total
    }

    fn _checksum_add_u32(acc: u16, v: u32) -> u16 {
        let bytes = v.to_be_bytes();
        let new_acc = Self::_checksum_add_bytes(acc, bytes[0], bytes[1]);
        Self::_checksum_add_bytes(new_acc, bytes[2], bytes[3])
    }

    fn _checksum_add_bytes(acc: u16, msb: u8, lsb: u8) -> u16 {
        Self::_checksum_add(acc, u16::from_be_bytes([msb, lsb]))
    }

    fn _checksum_self(&self) -> u16 {
        let mut total = 0;
        total = Self::_checksum_add_bytes(
            total,
            (self.version << 4) + self.ihl,
            (
                (self.precedence << 5) +
                bool_to_bit!(4, self.delay) +
                bool_to_bit!(3, self.throughput) +
                bool_to_bit!(2, self.reliability) +
                self.reserved_0
            )
        );

        total = Self::_checksum_add(total, self.total_length);
        total = Self::_checksum_add(total, self.identification);
        total = Self::_checksum_add(
            total,
            (
                bool_to_bit!(15, self.reserved_1) +
                bool_to_bit!(14, self.dont_fragment) +
                bool_to_bit!(13, self.more_fragments) +
                (self.fragment_offset & 0b0001_1111_1111_1111)
            )
        );

        total = Self::_checksum_add_bytes(total, self.ttl, self.proto);
        // Ignore Checksum

        total = Self::_checksum_add_u32(total, self.source_addr);
        total = Self::_checksum_add_u32(total, self.dest_addr);

        for byte_pair in self.options.chunks(2) {
            total = Self::_checksum_add_bytes(total, byte_pair[0], byte_pair[1]);
        };

        total
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn ihl(&self) -> u8 {
        self.ihl
    }

    pub fn precedence(&self) -> u8 {
        self.precedence
    }

    pub fn set_precedence(&mut self, precedence: u8) {
        if precedence > 7 {
            panic!("ipv4 header precedence must be between 0 and 7.");
        }
        
        self.precedence = precedence;
        self.generate_checksum();
    }

    pub fn dtr(&self) -> (bool, bool, bool) {
        (self.delay, self.throughput, self.reliability)
    }

    pub fn set_dtr(&mut self, delay: Option<bool>, throughput: Option<bool>, reliability: Option<bool>) {
        set_if_some!(self.delay, delay);
        set_if_some!(self.throughput, throughput);
        set_if_some!(self.reliability, reliability);

        self.generate_checksum();
    }

    pub fn total_length(&self) -> u16 {
        self.total_length
    }

    pub fn identification(&self) -> u16 {
        self.identification
    }

    pub fn set_identification(&mut self, identification: u16) {
        self.identification = identification;
        self.generate_checksum();
    }

    pub fn flags(&self) -> (bool, bool) {
        (self.dont_fragment, self.more_fragments)
    }

    pub fn set_flags(&mut self, dont_fragment: Option<bool>, more_fragments: Option<bool>) {
        set_if_some!(self.dont_fragment, dont_fragment);
        set_if_some!(self.more_fragments, more_fragments);

        self.generate_checksum();
    }

    pub fn fragment_offset(&self) -> u16 {
        self.fragment_offset
    }

    pub fn set_fragment_offset(&mut self, fragment_offset: u16) {
        if fragment_offset > 8191 {     // Biggest 13 bit
            panic!("ipv4 header precedence must be between 0 and 8191.");
        }

        self.fragment_offset = fragment_offset;
        self.generate_checksum();
    }

    pub fn ttl(&self) -> u8 {
        self.ttl
    }

    pub fn set_ttl(&mut self, ttl: u8) {
        self.ttl = ttl;
        self.generate_checksum();
    }

    pub fn proto(&self) -> u8 {
        self.proto
    }

    pub fn set_proto(&mut self, proto: u8) {
        self.proto = proto;
        self.generate_checksum();
    }

    pub fn source_addr(&self) -> u32 {
        self.source_addr
    }

    pub fn dest_addr(&self) -> u32 {
        self.dest_addr
    }

    pub fn set_dest_addr(&mut self, addr: u32) {
        self.dest_addr = addr;
        self.generate_checksum();
    }

    pub fn checksum(&self) -> u16 {
        self.checksum
    }

    fn generate_checksum(&mut self) {
        self.checksum = !self._checksum_self()
    }

    pub fn validate_checksum(&self) -> bool {
        let calculated = Self::_checksum_add(
            self._checksum_self(),
            self.checksum
        );

        !calculated == 0
    }

    pub fn get_byte_len(&self) -> usize {
        4 * self.ihl as usize
    }

    pub fn write_slice(&self, value: &mut [u8]) -> usize {
        value[0] = (self.version << 4) + self.ihl;
        value[1] = (
            (self.precedence << 5) +
            bool_to_bit!(4, self.delay) +
            bool_to_bit!(3, self.throughput) +
            bool_to_bit!(2, self.reliability) +
            self.reserved_0
        );

        let total_len_bytes = self.total_length.to_be_bytes();
        value[2] = total_len_bytes[0];
        value[3] = total_len_bytes[1];

        let idenitification_bytes = self.identification.to_be_bytes();
        value[4] = idenitification_bytes[0];
        value[5] = idenitification_bytes[1];

        let fragment_bits = self.fragment_offset.to_be_bytes();
        value[6] = (
            bool_to_bit!(7, self.reserved_1) +
            bool_to_bit!(6, self.dont_fragment) +
            bool_to_bit!(5, self.more_fragments) +
            (fragment_bits[0] & 0b0001_1111)
        );

        value[7] = fragment_bits[1];
        value[8] = self.ttl;
        value[9] = self.proto;

        let checksum_bytes = self.checksum.to_be_bytes();
        value[10] = checksum_bytes[0];
        value[11] = checksum_bytes[1];

        let srcaddr_bytes = self.source_addr.to_be_bytes();
        for (i, b) in srcaddr_bytes.iter().enumerate() {
            value[12 + i] = *b;
        }

        let dstaddr_bytes = self.dest_addr.to_be_bytes();
        for (i, b) in dstaddr_bytes.iter().enumerate() {
            value[16 + i] = *b;
        }

        for (i, b) in self.options.iter().enumerate() {
            value[20 + i] = *b;
        }

        self.get_byte_len()
    }

    pub fn from_slice(value: &[u8]) -> Option<Self> {
        if value.len() < 20 {
            return None;
        }

        let ihl = value[0] & 0x0f;
        let num_bytes = (ihl * 4) as usize;
        if value.len() < num_bytes {
            return None
        }

        Some(Self {
            version: (value[0] & 0xf0) >> 4,
            ihl,

            precedence:     (value[1] & 0b1110_0000) >> 5,
            delay:          (value[1] & 0b0001_0000) > 0,
            throughput:     (value[1] & 0b0000_1000) > 0,
            reliability:    (value[1] & 0b0000_0100) > 0,
            reserved_0:     (value[1] & 0b0000_0011),

            total_length: u16::from_be_bytes([value[2], value[3]]),
            identification: u16::from_be_bytes([value[4], value[5]]),

            reserved_1:     (value[6] & 0b1000_0000) > 0,
            dont_fragment:  (value[6] & 0b0100_0000) > 0,
            more_fragments: (value[6] & 0b0010_0000) > 0,

            fragment_offset: u16::from_be_bytes([value[6] & 0b0001_1111, value[7]]),

            ttl: value[8],
            proto: value[9],
            checksum: u16::from_be_bytes([value[10], value[11]]),

            source_addr: u32::from_be_bytes([value[12], value[13], value[14], value[15]]),
            dest_addr: u32::from_be_bytes([value[16], value[17], value[18], value[19]]),

            options: value[20..num_bytes].to_vec(),
        })
    }
}

impl Default for Ipv4Header {
    fn default() -> Self {
        let mut hdr = Self {
            version: 4,
            ihl: 5,
            precedence: 0,
            delay: false,
            throughput: false,
            reliability: false,
            reserved_0: 0,
            total_length: 20,
            identification: 0,
            reserved_1: false,
            dont_fragment: false,
            more_fragments: false,
            fragment_offset: 0,
            ttl: 64,
            proto: 6,
            checksum: 0,
            source_addr: 0,
            dest_addr: 0,
            options: vec![],
        };

        hdr.generate_checksum();
        hdr
    }
}

#[test]
fn test_serde() {
    let hdr = Ipv4Header {
        version: 4,
        ihl: 5,
        precedence: 0,
        delay: false,
        throughput: false,
        reliability: false,
        reserved_0: 0,
        total_length: 84,
        identification: 37931,
        reserved_1: false,
        dont_fragment: true,
        more_fragments: false,
        fragment_offset: 0,
        ttl: 64,
        proto: 1,
        checksum: 31579,
        source_addr: 2887778305,
        dest_addr: 2130706433,
        options: vec![],
    };

    let mut bytes = vec![0u8; hdr.get_byte_len()];
    hdr.write_slice(&mut bytes);

    match Ipv4Header::from_slice(&bytes) {
        Some(newhdr) => {
            assert!(newhdr.validate_checksum());
            assert_eq!(newhdr.checksum(), hdr.checksum());
        },
        None => panic!("parsing ipv4 header bytes failed")
    }
}

#[test]
fn test_bool_to_bit() {
    assert_eq!(1 << 0, bool_to_bit!(0, true));
    assert_eq!(1 << 1, bool_to_bit!(1, true));
    assert_eq!(1 << 2, bool_to_bit!(2, true));
    assert_eq!(1 << 3, bool_to_bit!(3, true));
    assert_eq!(1 << 4, bool_to_bit!(4, true));
    assert_eq!(1 << 5, bool_to_bit!(5, true));
    assert_eq!(1 << 6, bool_to_bit!(6, true));
    assert_eq!(1 << 7, bool_to_bit!(7, true));
}