use super::super::ascii85::decode;
use anyhow::{anyhow, ensure, Result};
use std::convert::TryInto;
use std::net::Ipv4Addr;

fn read_as_u16(bytes: &[u8]) -> Result<Vec<u16>> {
    let words = bytes
        .chunks_exact(2)
        .map(|chunk| {
            let word: Result<[u8; 2]> = chunk
                .try_into()
                .map_err(|_| anyhow!("couldn't fit {} bytes into 16 bit word", chunk.len()));
            word.map(u16::from_be_bytes)
        })
        .collect::<Result<Vec<u16>>>()?;

    Ok(words)
}

fn read_as_u16_unchecked(bytes: &[u8]) -> Vec<u16> {
    let err_msg = format!("failed to read byte array as Vec<u16> {:?}", bytes);
    read_as_u16(bytes).expect(&err_msg)
}

#[derive(Debug)]
struct EmptyUdpPacket(UdpPacket);

impl EmptyUdpPacket {
    fn set_data(mut self, data: &[u8]) -> UdpPacket {
        self.0.data = data.to_vec();
        self.0
    }

    fn len(&self) -> u16 {
        self.0.udp_header.length - 8
    }
}

#[derive(Debug)]
struct UdpPacket {
    ip_header: Ipv4Header,
    udp_psuedo_header: UdpPseudoHeader,
    udp_header: UdpHeader,
    data: Vec<u8>,
}

impl UdpPacket {
    fn parse_headers(bytes: [u8; 28]) -> Result<EmptyUdpPacket> {
        let ip_header = Ipv4Header::from_bytes(&bytes[0..20])?;
        let (udp_psuedo_header, udp_header) = parse_udp_headers(&ip_header, &bytes[20..28])?;

        Ok(EmptyUdpPacket(UdpPacket {
            ip_header,
            udp_psuedo_header,
            udp_header,
            data: Vec::new(),
        }))
    }

    fn valid_ip_checksum(&self) -> bool {
        self.ip_header.valid_checksum()
    }

    fn valid_udp_checksum(&self) -> bool {
        let mut bytes: Vec<u8> = Vec::with_capacity(20);

        // https://en.wikipedia.org/wiki/User_Datagram_Protocol#IPv4_pseudo_header

        // Source Address
        // Destination Address
        // Zeroes
        // Protocol
        // UDP Length

        // Source Port
        // Destination Port
        // Length
        // Checksum

        // Data

        bytes.append(&mut self.udp_psuedo_header.source_address.octets().to_vec());
        bytes.append(&mut self.udp_psuedo_header.destination_address.octets().to_vec());
        bytes.push(0x00);
        bytes.push(self.udp_psuedo_header.protocol);
        bytes.append(&mut self.udp_psuedo_header.udp_length.to_be_bytes().to_vec());

        bytes.append(&mut self.udp_header.source_port.to_be_bytes().to_vec());
        bytes.append(&mut self.udp_header.destination_port.to_be_bytes().to_vec());
        bytes.append(&mut self.udp_header.length.to_be_bytes().to_vec());
        bytes.append(&mut self.udp_header.checksum.to_be_bytes().to_vec());

        bytes.extend_from_slice(&self.data);

        if bytes.len() % 2 != 0 {
            bytes.push(0)
        }

        read_as_u16_unchecked(&bytes)
            .iter()
            .fold(0xffff, |sum, &next| ones_complement_sum(sum, next))
            == 0xffff
    }

    fn valid_checksums(&self) -> bool {
        let valid_udp = self.valid_udp_checksum();
        let valid_ip = self.valid_ip_checksum();

        valid_ip && valid_udp
    }
}

#[test]
fn test_udp_parse() -> Result<()> {
    // ip cksum 0c741
    //udp cksum xcc52

    let bytes: [u8; 40] = [
        // ip header
        0x45, 0x00, // stuff i can ignore :)
        0x00, 0x28, // total length (40)
        0xb5, 0x81, 0x00, 0x00, 0x40, // stuff i can ignore :)
        0x11, // protocol (17 -> UDP)
        0xc7, 0x41, // ip cksum
        0x7f, 0x00, 0x00, 0x01, // src addr (127.0.0.1)
        0x7f, 0x00, 0x00, 0x01, // dest addr (127.0.0.1)
        // no options

        // udp header
        0xc9, 0x64, // src port (51556)
        0x1f, 0xbd, // dest port (8125)
        0x00, 0x14, // udp length (header + data, 8 + 12 = 20)
        0xcc, 0x52, // udp chksum
        // data
        0x72, 0x75, 0x73, 0x74, 0x20, 0x69, 0x73, 0x20, 0x63, 0x6f, 0x6f,
        0x6c, // (rust is cool)
    ];

    let header: [u8; 28] = bytes[0..28].try_into()?;
    let data: [u8; 12] = bytes[28..].try_into()?;

    let packet = UdpPacket::parse_headers(header)?.set_data(&data);

    assert_eq!(packet.ip_header.source, Ipv4Addr::new(127, 0, 0, 1));
    assert_eq!(packet.ip_header.destination, Ipv4Addr::new(127, 0, 0, 1));
    assert!(packet.ip_header.valid_checksum());

    assert_eq!(packet.udp_psuedo_header.source_address, Ipv4Addr::LOCALHOST);
    assert_eq!(
        packet.udp_psuedo_header.destination_address,
        Ipv4Addr::LOCALHOST
    );
    assert_eq!(packet.udp_psuedo_header.protocol, 17);
    assert_eq!(packet.udp_psuedo_header.udp_length, 20);

    assert_eq!(packet.udp_header.source_port, 51556);
    assert_eq!(packet.udp_header.destination_port, 8125);
    assert_eq!(packet.udp_header.length, 20);

    assert!(packet.valid_udp_checksum());
    assert!(packet.valid_checksums()); // already checked individually, but make sure this method works

    Ok(())
}

#[derive(Debug)]
// an IPv4 packet has much more info than this, but for this we only care about these fields
struct Ipv4Header {
    source: Ipv4Addr,
    destination: Ipv4Addr,
    checksum: u16,
    words: Vec<u16>, // todo protocol 0x11
}

impl Ipv4Header {
    fn from_bytes(bytes: &[u8]) -> Result<Ipv4Header> {
        ensure!(
            bytes.len() == 20,
            anyhow!("Invalid header length={}", bytes.len())
        );

        let checksum: [u8; 2] = bytes[10..12].try_into()?;
        let src: [u8; 4] = bytes[12..16].try_into()?;
        let dst: [u8; 4] = bytes[16..20].try_into()?;

        let source = Ipv4Addr::from(src);
        let destination = Ipv4Addr::from(dst);

        Ok(Ipv4Header {
            source,
            destination,
            checksum: u16::from_be_bytes(checksum),
            words: read_as_u16(bytes)?,
        })
    }

    fn valid_checksum(&self) -> bool {
        self.words
            .iter()
            .fold(0xffff, |sum, &next| ones_complement_sum(sum, next))
            == 0xffff
    }
}

#[test]
fn test_from_bytes() -> Result<()> {
    // a random packet from tcpdump
    let packet = [
        0x45, 0x00, 0x00, 0xd0, 0xb4, 0x2a, 0x40, 0x00, 0x40, 0x06, 0xc2, 0xd8, 0xac, 0x18, 0xba,
        0xf2, 0xac, 0x18, 0xb0, 0x01,
    ];
    let out = Ipv4Header::from_bytes(&packet)?;
    assert_eq!(out.source, Ipv4Addr::new(172, 24, 186, 242));
    assert_eq!(out.destination, Ipv4Addr::new(172, 24, 176, 1));

    Ok(())
}
#[derive(Debug)]
struct UdpPseudoHeader {
    source_address: Ipv4Addr,
    destination_address: Ipv4Addr,
    protocol: u8,
    udp_length: u16,
}
#[derive(Debug)]
struct UdpHeader {
    source_port: u16,
    destination_port: u16,
    length: u16, // wtf two lengths (https://stackoverflow.com/a/26356487)
    checksum: u16,
}

fn parse_udp_headers(ip_header: &Ipv4Header, bytes: &[u8]) -> Result<(UdpPseudoHeader, UdpHeader)> {
    ensure!(
        bytes.len() == 8,
        anyhow!("Invalid header length={}", bytes.len())
    );

    let src: [u8; 2] = bytes[..2].try_into()?;
    let dest: [u8; 2] = bytes[2..4].try_into()?;
    let length: [u8; 2] = bytes[4..6].try_into()?;
    let checksum: [u8; 2] = bytes[6..8].try_into()?;

    let psuedo_header = UdpPseudoHeader {
        source_address: ip_header.source,
        destination_address: ip_header.destination,
        protocol: 0x11, // todo
        udp_length: u16::from_be_bytes(length),
    };

    let header = UdpHeader {
        source_port: u16::from_be_bytes(src),
        destination_port: u16::from_be_bytes(dest),
        length: u16::from_be_bytes(length),
        checksum: u16::from_be_bytes(checksum),
    };

    Ok((psuedo_header, header))
}

fn ones_complement_sum(x: u16, y: u16) -> u16 {
    let sum: u32 = x as u32 + y as u32;

    let low_word = (sum & 0xffff) as u16;
    low_word + ((sum >> 16) as u16)
}

#[test]
fn test_ones_complement_sum() {
    //   0001 0110     22
    // + 0000 0011      3
    // ===========   ====
    //   0001 1001     25z

    let x = 0b_0000_0000_0001_0110;
    let y = 0b_0000_0000_0000_0011;
    assert_eq!(ones_complement_sum(x, y), 0b_0000_0000_0001_1001);

    //   1111 1110     -1   (254)
    // + 0000 0001      1
    // ===========   ====
    //   1111 1111     -0   (lol)

    let x = 0b_0000_0000_1111_1110;
    let y = 0b_0000_0000_0000_0001;
    assert_eq!(ones_complement_sum(x, y), 0b_0000_0000_1111_1111);

    //   1111 1110     -1   (254)
    // + 0000 0011      3
    // ===========   ====
    // 1 0000 0001
    // \________
    //          \
    // + 0000 0001
    // ===========   ====
    //   0000 0010      2

    let x = 0b_1111_1111_1111_1110;
    let y = 0b_0000_0000_0000_0011;
    assert_eq!(ones_complement_sum(x, y), 0b_0000_0000_0000_0010);
}

fn parse_and_filter_packets(bytes: &[u8]) -> Result<Vec<UdpPacket>> {
    // take 28 bytes
    // parse a udp packet
    // let n = length
    // take n bytes
    // set data
    let mut idx = 0;
    let mut packets = Vec::new();

    while idx < bytes.len() {
        if idx > bytes.len() {
            break;
        }

        let data_start = idx + 28;

        let header_data: [u8; 28] = bytes[idx..data_start].try_into()?;
        let header = UdpPacket::parse_headers(header_data)?;
        let data_end = data_start + header.len() as usize;

        if data_start > bytes.len() || data_end > bytes.len() {
            eprintln!("Ran out of data while processing header= {:#?}. idx={:?}, data_start={:?}, data_end={:?}, bytes.len={:?}", header, idx, data_start, data_end, bytes.len());
            eprintln!("header bytes: \n {:?}", header_data);
            break;
        }

        packets.push(header.set_data(&bytes[data_start..data_end]));

        idx = data_end;
    }

    Ok(packets
        .into_iter()
        .filter(|packet| {
            packet.valid_checksums()
                && packet.ip_header.source == Ipv4Addr::new(10, 1, 1, 10)
                && packet.ip_header.destination == Ipv4Addr::new(10, 1, 1, 200)
                && packet.udp_header.destination_port == 42069
        })
        .collect())
}

pub fn run(bytes: &[u8]) -> Result<Vec<u8>> {
    let packets = parse_and_filter_packets(&decode(bytes)?)?;

    Ok(packets
        .into_iter()
        .map(|packet| packet.data)
        .flatten()
        .collect())
}
