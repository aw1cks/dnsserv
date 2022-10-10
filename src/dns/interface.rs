type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

use std::net::UdpSocket;

use crate::dns::buffer::BytePacketBuffer;
use crate::dns::packet::DnsPacket;
use crate::dns::result::ResultCode;
use crate::dns::querytype::QueryType;
use crate::dns::question::DnsQuestion;

fn lookup(qname: &str, qtype: QueryType) -> Result<DnsPacket> {
    let server = ("8.8.8.8", 53);
    let sock = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = DnsPacket::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet.questions.push(
        DnsQuestion::new(qname.to_string(), qtype)
    );

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    sock.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    sock.recv_from(&mut res_buffer.buf)?;

    DnsPacket::from_buffer(&mut res_buffer)
}

pub fn handle_inbound_query(sock: &UdpSocket) -> Result<()> {
    let mut req_buffer = BytePacketBuffer::new();

    let(_, src) = sock.recv_from(&mut req_buffer.buf)?;

    let mut request = DnsPacket::from_buffer(&mut req_buffer)?;

    let mut packet = DnsPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;

    if let Some(question) = request.questions.pop() {
        println!("Received query: {:?}", question);

        if let Ok(result) = lookup(&question.name, question.qtype) {
            packet.questions.push(question);
            packet.header.rescode = result.header.rescode;

            for rec in result.answers {
                println!("Answer: {:?}", rec);
                packet.answers.push(rec);
            }
            for rec in result.authorities {
                println!("Authority: {:?}", rec);
                packet.authorities.push(rec);
            }
            for rec in result.resources {
                println!("Resource: {:?}", rec);
                packet.resources.push(rec);
            }
        } else {
            packet.header.rescode = ResultCode::SERVFAIL;
        }
    } else {
        packet.header.rescode = ResultCode::FORMERR;
    }

    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;

    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;

    sock.send_to(data, src)?;

    Ok(())
}

