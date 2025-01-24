use messages::Message;
use rand::seq::IndexedRandom;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Wrap;
use wg_2024::packet::Nack;
use wg_2024::packet::NackType;
use wg_2024::packet::Packet;
use wg_2024::packet::PacketType;

use crate::utilities::theme::*;

pub fn message_table_row(message: &Message, finished_sending: bool) -> Row {
    let source: Span = Span::styled(format!("{}", message.source_id), Style::new());
    let sess_id: Span = Span::styled(format!("{}", message.session_id), Style::new());

    let mut mtype: Span;
    let rtype: Span;

    let ptype_style: Style = Style::new();

    match &message.content {
        messages::MessageType::Request(request_type) => {
            mtype = Span::styled("RQS", ptype_style.bg(MESSAGE_REQUEST_COLOR));
            match request_type {
                messages::RequestType::MediaRequest(_) => {
                    rtype = Span::from("MED".to_string());
                    //    rtype = Span::from(format!("media request: {:?}", media_request));
                }
                messages::RequestType::TextRequest(_) => {
                    rtype = Span::from("TXT".to_string());
                    //    rtype = Span::from(format!("text request: {:?}", text_request));
                }
                messages::RequestType::ChatRequest(_) => {
                    rtype = Span::from("CHT".to_string());
                    //    rtype = Span::from(format!("chat request: {:?}", chat_request));
                }
                messages::RequestType::DiscoveryRequest(_) => {
                    rtype = Span::from("DSC".to_string());
                    //    rtype = Span::from("discovery".to_string());
                }
            }
        }
        messages::MessageType::Response(response_type) => {
            mtype = Span::styled("RSP", ptype_style.bg(MESSAGE_RESPONSE_COLOR));
            match response_type {
                messages::ResponseType::TextResponse(text_response) => {
                    rtype = Span::from("TXT".to_string());
                }
                messages::ResponseType::MediaResponse(media_response) => {
                    rtype = Span::from("MED".to_string());
                }
                messages::ResponseType::ChatResponse(chat_response) => {
                    rtype = Span::from("CHT".to_string());
                }
                messages::ResponseType::DiscoveryResponse(server_type) => {
                    //rtype = Span::from(format!("discovery response: {:?}", server_type));
                    rtype = Span::from("DSC".to_string());
                }
            }
        }
    }

    if !finished_sending {
        mtype = mtype.bg(MESSAGE_SENDING_COLOR).slow_blink();
    }
    Row::new(vec![rtype, mtype, source, sess_id])
}

pub fn packet_table_row(packet: &Packet) -> Row {
    let sess_id: Span = Span::styled(format!("{}", packet.session_id), Style::new());

    let src: Span;
    let dest: Span;
    if packet.routing_header.hop_index < packet.routing_header.hops.len() {
        let from = packet.routing_header.hops[packet.routing_header.hop_index - 1];
        let to = packet.routing_header.hops[packet.routing_header.hop_index];
        src = Span::styled(format!("{}", from), Style::new());
        dest = Span::styled(format!("{}", to), Style::new());
    } else {
        src = Span::styled("Err".to_string(), Style::new());
        dest = Span::styled("Err".to_string(), Style::new());
    }

    let ptype: Span;
    let ptype_style: Style = Style::new();

    let depends_on_type: Span;

    match &packet.pack_type {
        PacketType::MsgFragment(fragment) => {
            ptype = Span::styled("FRG", ptype_style.bg(PACKET_FRAGMENT_COLOR));
            depends_on_type = Span::from(format!(
                "#{}/{} size({})",
                fragment.fragment_index, fragment.total_n_fragments, fragment.length
            ));
        }
        PacketType::Nack(nack) => {
            ptype = Span::styled("NCK", ptype_style.bg(PACKET_NACK_COLOR));
            depends_on_type = Span::from(match nack.nack_type {
                NackType::ErrorInRouting(id) => format!("ErrorInRouting: neigbor({})", id),
                NackType::DestinationIsDrone => "DestinationIsDrone".to_string(),
                NackType::Dropped => format!("Dropped({})", nack.fragment_index),
                NackType::UnexpectedRecipient(id) => format!("UnexpectedRecipient({})", id),
            });
        }
        PacketType::Ack(ack) => {
            ptype = Span::styled("ACK", ptype_style.bg(PACKET_ACK_COLOR));
            depends_on_type = Span::from(format!("Fragment_idx({})", ack.fragment_index))
        }
        PacketType::FloodRequest(flood_request) => {
            ptype = Span::styled("FRQ", ptype_style.bg(PACKET_FLOOD_REQUEST_COLOR));
            depends_on_type = Span::from(format!(
                "id({}) initiator({})",
                flood_request.flood_id, flood_request.initiator_id
            ));
        }
        PacketType::FloodResponse(flood_response) => {
            ptype = Span::styled("FRS", ptype_style.bg(PACKET_FLOOD_RESPONSE_COLOR));
            depends_on_type = Span::from(format!("flood_id({})", flood_response.flood_id));
        }
    }

    Row::new(vec![ptype, sess_id, src, dest, depends_on_type])
}

pub fn packet_detail(packet: &Packet) -> Paragraph {
    let routing = Line::from(format!(
        "SID: {} Hops: {}",
        packet.session_id, packet.routing_header
    ));
    let theader: Line;
    let depends_on_type: Line;

    let mut res = Text::from(routing);
    match &packet.pack_type {
        PacketType::MsgFragment(fragment) => {
            theader = Line::styled("Fragment", Style::new().bg(PACKET_FRAGMENT_COLOR));
            depends_on_type = Line::from(format!(
                "Index:{}/{} Size:{}",
                fragment.fragment_index, fragment.total_n_fragments, fragment.length
            ));

            res.push_line(theader);
            res.push_line(depends_on_type);
            res.push_line("Data:");
            let p = Line::from(format!("{:?}", fragment.data));
            res.push_line(p);
        }
        PacketType::Nack(nack) => {
            theader = Line::styled("Nack", Style::new().bg(PACKET_NACK_COLOR));
            depends_on_type = Line::from(match nack.nack_type {
                NackType::ErrorInRouting(id) => format!("Type: ErrorInRouting(id:{})", id),
                NackType::DestinationIsDrone => "Type: DestinationIsDrone".to_string(),
                NackType::Dropped => format!("Type: Dropped(id:{})", nack.fragment_index),
                NackType::UnexpectedRecipient(id) => {
                    format!("Type: UnexpectedRecipient(id:{})", id)
                }
            });
            res.push_line(theader);
            res.push_line(depends_on_type);
        }
        PacketType::Ack(ack) => {
            theader = Line::styled("(Qu)Ack", Style::new().bg(PACKET_ACK_COLOR));
            depends_on_type = Line::from(format!("Fragment index:{}", ack.fragment_index));
            res.push_line(theader);
            res.push_line(depends_on_type);
        }
        PacketType::FloodRequest(flood_request) => {
            theader = Line::styled("Flood Request", Style::new().bg(PACKET_FLOOD_REQUEST_COLOR));
            depends_on_type = Line::from(format!(
                "Id:{} Initiator:{}",
                flood_request.flood_id, flood_request.initiator_id
            ));
            res.push_line(theader);
            res.push_line(depends_on_type);
            res.push_line("Path trace:");
            let p = Line::from(format!("{:?}", flood_request.path_trace));
            res.push_line(p);
        }
        PacketType::FloodResponse(flood_response) => {
            theader = Line::styled(
                "Flood Response",
                Style::new().bg(PACKET_FLOOD_RESPONSE_COLOR),
            );
            depends_on_type = Line::from(format!("Flood id:{}", flood_response.flood_id));
            res.push_line(theader);
            res.push_line(depends_on_type);
            res.push_line("Path trace:");
            let p = Line::from(format!("{:?}", flood_response.path_trace));
            res.push_line(p);
        }
    }
    Paragraph::new(res).wrap(Wrap { trim: true })
}
