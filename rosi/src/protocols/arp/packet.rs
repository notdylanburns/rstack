use crate::common::{DeserialiseError, Serialise, serialise_fields};

use crate::protocols::ethernet;
use super::enums::{
    Htype, Operation,
    HardwareAddress, ProtocolAddress,
};

#[derive(Debug)]
pub struct Packet {
    htype: Htype,
    ptype: ethernet::EtherType,
    hlen: u8,
    plen: u8,
    operation: Operation,
    sha: HardwareAddress,
    spa: ProtocolAddress,
    tha: HardwareAddress,
    tpa: ProtocolAddress,
}

impl Packet {
    fn new(operation: Operation, sha: HardwareAddress, spa: ProtocolAddress, tha: HardwareAddress, tpa: ProtocolAddress) -> Option<Self> {
        if
            core::mem::discriminant(&sha) != core::mem::discriminant(&tha) ||
            core::mem::discriminant(&spa) != core::mem::discriminant(&tpa)
        {
            None
        } else {
            Some(Self {
                htype: sha.addr_type(),
                ptype: spa.addr_type(),
                hlen: sha.byte_length() as u8,
                plen: spa.byte_length() as u8,
                operation,
                sha,
                spa,
                tha,
                tpa,
            })
        }
    }

    pub fn request(sha: HardwareAddress, spa: ProtocolAddress, tha: HardwareAddress, tpa: ProtocolAddress) -> Option<Self> {
        Self::new(Operation::Request, sha, spa, tha, tpa)
    }

    pub fn response(sha: HardwareAddress, spa: ProtocolAddress, tha: HardwareAddress, tpa: ProtocolAddress) -> Option<Self> {
        Self::new(Operation::Response, sha, spa, tha, tpa)
    }

    crate::util::getter!(htype: Htype);
    crate::util::getter!(ptype: ethernet::EtherType);
    crate::util::getter!(hlen: u8);
    crate::util::getter!(plen: u8);
    crate::util::getter!(operation: Operation);
    crate::util::getter!(sha: HardwareAddress);
    crate::util::getter!(spa: ProtocolAddress);
    crate::util::getter!(tha: HardwareAddress);
    crate::util::getter!(tpa: ProtocolAddress);
}

impl Serialise for Packet {
    #[inline]
    fn byte_length(&self) -> usize {
        self.htype.byte_length() + self.ptype.byte_length() +
        self.hlen.byte_length() + self.plen.byte_length() +
        self.operation.byte_length() +
        (self.hlen as usize) * 2 +
        (self.plen as usize) * 2
    }

    fn serialise(&self, buf: &mut [u8]) -> usize {
        serialise_fields!(
            start=0, buf=buf,
            self.htype, self.ptype,
            self.hlen, self.plen,
            self.operation,
            self.sha, self.spa,
            self.tha, self.tpa,
        )
    }

    fn deserialise(buf: &[u8]) -> Result<Self, DeserialiseError>
    where Self: Sized
    {
        let htype = Htype::deserialise(buf)?;
        let index = htype.byte_length();

        let ptype = ethernet::EtherType::deserialise(&buf[index..])?;
        let index = index + ptype.byte_length();

        let hlen = u8::deserialise(&buf[index..])?;
        let plen = u8::deserialise(&buf[index + 1..])?;
        let operation = Operation::deserialise(&buf[index + 2..])?;
        if let Operation::Unknown(v) = operation {
            return Err(DeserialiseError::Heap(format!("unknown operation: {v}")))
        }

        let index = index + operation.byte_length() + 2;

        let sha = HardwareAddress::from_bytes(htype, &buf[index..])?;
        let spa = ProtocolAddress::from_bytes(ptype, &buf[index + sha.byte_length()..])?;

        let index = index + (hlen as usize) + (plen as usize);

        let tha = HardwareAddress::from_bytes(htype, &buf[index..])?;
        let tpa = ProtocolAddress::from_bytes(ptype, &buf[index + sha.byte_length()..])?;

        Ok(Self {
            htype, ptype,
            hlen, plen,
            operation,
            sha, spa,
            tha, tpa
        })
    }
}

impl core::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ARP {} (Hardware: {}, Protocol: {}) - SHA: {}, SPA: {}, THA: {}, TPA: {}",
            self.operation,
            self.htype, self.ptype,
            self.sha, self.spa,
            self.tha, self.tpa,
        )
    }
}

#[test]
#[allow(unused)]
fn test_arp_multiple_new() {
    use crate::common::{
        Address, Serialise,
    };

    use crate::common::address::{
        Ipv4Address, Ipv6Address,
        MacAddress
    };

    let arp = Packet::request(
        MacAddress::from_hex("00:11:5d:48:2f:53").unwrap().into(),
        Ipv4Address::from([192, 168, 0, 1]).into(),
        MacAddress::default().into(),
        Ipv4Address::from([192, 168, 1, 1]).into(),
    ).unwrap();

    println!("{arp}");

    let mut bytes = vec![0u8; arp.byte_length()];
    arp.serialise(&mut bytes);

    let new_arp = Packet::deserialise(&bytes).unwrap();
    println!("{new_arp}")
}