use crate::common::{MacAddress, DeserialiseError, Serialise, Layer};
use super::ethertype::EtherType;

#[derive(Debug)]
struct FrameHeader {
    mac_destination: MacAddress,
    mac_source: MacAddress,

    ethertype: EtherType,
    tpid: Option<EtherType>,
    tci: u16,   // Presence dependent on tpid
}

impl Serialise for FrameHeader {
    #[inline]
    fn byte_length(&self) -> usize {
        self.mac_destination.byte_length() +
        self.mac_source.byte_length() +
        self.ethertype.byte_length() +
        match self.tpid {
            Some(tpid) => tpid.byte_length() + 2,
            None => 0,
        }
    }

    fn serialise(&self, buf: &mut [u8]) -> usize {
        let mut index = 0usize;

        index += self.mac_destination.serialise(&mut buf[index..]);
        index += self.mac_source.serialise(&mut buf[index..]);

        if let Some(tpid) = self.tpid {
            index += tpid.serialise(&mut buf[index..]);
            index += self.tci.serialise(&mut buf[index..]);
        };

        index += self.ethertype.serialise(&mut buf[index..]);
        index
    }

    fn deserialise(buf: &[u8]) -> Result<Self, DeserialiseError> {
        let mut index = 0;
        let mac_destination = MacAddress::deserialise(&buf[index..])?;
        index += mac_destination.byte_length();

        let mac_source = MacAddress::deserialise(&buf[index..])?;
        index += mac_source.byte_length();

        let ethertype = EtherType::deserialise(&buf[index..])?;
        let (ethertype, tpid, tci) = match ethertype {
            EtherType::ServiceVlanTag | EtherType::VlanTaggedFrame => {
                let tci = u16::deserialise(&buf[index..])?;
                let new_ethertype = EtherType::deserialise(&buf[index..])?;

                (new_ethertype, Some(ethertype), tci)
            }
            _ => (ethertype, None, 0),
        };

        Ok(Self {
            mac_destination,
            mac_source,
            ethertype,
            tpid,
            tci,
        })
    }
}

#[derive(Debug)]
pub struct Frame {
    header: FrameHeader,
    data: Vec<u8>,
    fcs: u32,
}

#[allow(dead_code)]
impl Frame {
    pub fn new(
        destination: MacAddress, source: MacAddress,
        ethertype: EtherType,
        data: Vec<u8>,
    ) -> Self {
        Self {
            header: FrameHeader {
                mac_destination: destination,
                mac_source: source,
                ethertype,
                tpid: None,
                tci: 0,
            },
            data,
            fcs: 0,
        }
    }

    pub fn new_vlan_tagged(
        destination: MacAddress, source: MacAddress,
        tpid: EtherType, tci: u16,
        ethertype: EtherType,
        data: Vec<u8>,
    ) -> Self {
        Self {
            header: FrameHeader {
                mac_destination: destination,
                mac_source: source,
                ethertype,
                tpid: Some(tpid),
                tci,
            },
            data,
            fcs: 0,
        }
    }

    pub fn destination(&self) -> MacAddress {
        self.header.mac_destination
    }

    pub fn source(&self) -> MacAddress {
        self.header.mac_source
    }

    pub fn ethertype(&self) -> EtherType {
        self.header.ethertype
    }

    pub fn tpid(&self) -> Option<EtherType> {
        self.header.tpid
    }

    pub fn tci(&self) -> Option<u16> {
        self.header.tpid.and(Some(self.header.tci))
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn fcs(&self) -> u32 {
        self.fcs
    }

    fn get_fcs(&self) -> u32 {
        todo!()
        /*
        polynomial = 0x4C11DB7
        initial_value = 0xFFFFFFFF
        */
    }
}

impl core::fmt::Display for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Frame: {} bytes", self.byte_length())?;
        writeln!(f, "-----------------")?;
        writeln!(f, "Destination: {}", self.header.mac_destination)?;
        writeln!(f, "Source:      {}", self.header.mac_source)?;
        writeln!(f, "EtherType:   {}", self.header.ethertype)?;
        writeln!(f, "FCS:         {:08x}", self.fcs)?;
        writeln!(f)?;
        if let Some(tpid) = self.header.tpid {
            writeln!(f, "VLAN Tagging:")?;
            writeln!(f, "-----------------")?;
            writeln!(f, "TPID:        {}", tpid)?;
            writeln!(f, "TCI:         {}", self.header.tci)?;
            writeln!(f)?;
        };

        if !self.data.is_empty() {
            writeln!(f, "Data:")?;
            writeln!(f, "-----------------")?;
            self.data.chunks(16).try_for_each(|chunk| {
                let mut line = String::with_capacity(16 * 3);
                chunk.iter().for_each(|d| line.push_str(&format!("{:02x} ", d)));
                writeln!(f, "{}", line)
            })?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Serialise for Frame {
    fn byte_length(&self) -> usize {
        self.header.byte_length()
        + self.data.len()
        // + self.fcs.byte_length()
    }

    fn serialise(&self, buf: &mut [u8]) -> usize {
        let index = 0;
        let index = index + self.header.serialise(&mut buf[index..]);
        let index = index + self.data.as_slice().serialise(&mut buf[index..]);
        index // + self.fcs.serialise(&mut buf[index..])
    }

    fn deserialise(buf: &[u8]) -> Result<Self, DeserialiseError> {
        let header = FrameHeader::deserialise(buf)?;

        let end_index = if let EtherType::PayloadLength(len) = header.ethertype {
            header.byte_length() + len as usize
        } else {
            // buf.len() - 4
            buf.len()
        };

        let data = buf[header.byte_length()..end_index].to_owned();
        // let fcs = u32::deserialise(&buf[end_index..])?;

        Ok(Self {
            header,
            data,
            fcs: 0 // fcs
        })
    }
}

impl Layer for Frame {
    fn wrap(&mut self, data: &dyn Serialise) {
        self.data = vec![0u8; data.byte_length()];
        data.serialise(self.data.as_mut_slice());
    }
}

#[test]
fn test_display_frame() {
    let frame = Frame::new_vlan_tagged(
        MacAddress::from_hex("fe:77:4d:96:d5:95").unwrap(),
        MacAddress::from_hex("33:33:00:00:00:02").unwrap(),
        EtherType::VlanTaggedFrame,
        0xdead,
        EtherType::Ipv4,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
    );

    print!("{frame}");
}