use rand::seq::IndexedRandom;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::Row;
use wg_2024::packet::Nack;
use wg_2024::packet::NackType;
use wg_2024::packet::Packet;
use wg_2024::packet::PacketType;

use crate::utilities::theme::*;
pub fn format_packet(packet: &Packet) -> Row {
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
                "flood_id({}) initiated_by({})",
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
