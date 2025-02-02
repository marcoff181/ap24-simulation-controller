use messages::Message;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Wrap;
use wg_2024::packet::NackType;
use wg_2024::packet::Packet;
use wg_2024::packet::PacketType;

use crate::utilities::theme::*;

pub fn message_table_row(message: &Message, finished_sending: bool) -> Row {
    let source: Span = Span::styled(format!("{}", message.source), Style::new());
    let sess_id: Span = Span::styled(format!("{}", message.session_id), Style::new());

    let mut mtype: Span;
    let rtype: Span;
    let debug: Span;

    let ptype_style: Style = Style::new();

    match &message.content {
        messages::MessageType::Request(request_type) => {
            mtype = Span::styled("RQS", ptype_style.bg(MESSAGE_REQUEST_COLOR));
            debug = Span::from(format!("{:?}", request_type));
            match request_type {
                messages::RequestType::TextRequest(_) => {
                    rtype = Span::from("TXT".to_string()).bg(TEXT_MSG);
                }
                messages::RequestType::MediaRequest(_) => {
                    rtype = Span::from("MED".to_string()).bg(MEDIA_MSG);
                }
                messages::RequestType::ChatRequest(_) => {
                    rtype = Span::from("CHT".to_string()).bg(CHAT_MSG);
                }
                messages::RequestType::DiscoveryRequest(_) => {
                    rtype = Span::from("DSC".to_string()).bg(DISCOVERY_MSG);
                }
            }
        }
        messages::MessageType::Response(response_type) => {
            mtype = Span::styled("RSP", ptype_style.bg(MESSAGE_RESPONSE_COLOR));
            debug = Span::from(format!("{:?}", response_type));
            match response_type {
                messages::ResponseType::TextResponse(_) => {
                    rtype = Span::from("TXT".to_string()).bg(TEXT_MSG);
                }
                messages::ResponseType::MediaResponse(_) => {
                    rtype = Span::from("MED".to_string()).bg(MEDIA_MSG);
                }
                messages::ResponseType::ChatResponse(_) => {
                    rtype = Span::from("CHT".to_string()).bg(CHAT_MSG);
                }
                messages::ResponseType::DiscoveryResponse(_) => {
                    rtype = Span::from("DSC".to_string()).bg(DISCOVERY_MSG);
                }
            }
        }
        messages::MessageType::Error(error_type) => {
            mtype = Span::styled("ERR", ptype_style.bg(MESSAGE_ERROR_COLOR));
            debug = Span::from(format!("{:?}", error_type));
            match error_type {
                messages::ErrorType::Unsupported(_) => {
                    rtype = Span::from("UNS".to_string()).bg(UNSUPPORTED_MSG);
                }
                messages::ErrorType::Unexpected(_) => {
                    rtype = Span::from("UNX".to_string()).bg(UNEXPECTED_MSG);
                }
            }
        }
    }

    if !finished_sending {
        mtype = mtype.bg(BG_COLOR).slow_blink();
    }
    Row::new(vec![rtype, mtype, source, sess_id, debug])
}

pub fn message_detail(message: &Message) -> Paragraph {
    let routing = Line::from(format!(
        "SID: {} source: {}",
        message.session_id, message.source
    ));
    let mtype: Span;
    let rtype: Span;
    let h2: Line;
    let mut opt: Line = Line::default();

    let mut res = Text::from(routing);
    match &message.content {
        messages::MessageType::Request(request_type) => {
            mtype = Span::styled("Request", Style::new().bg(MESSAGE_REQUEST_COLOR));
            match request_type {
                messages::RequestType::TextRequest(text_request) => {
                    rtype = Span::styled("Text", Style::new().bg(TEXT_MSG));
                    match text_request {
                        messages::TextRequest::TextList => {
                            h2 = Line::from("Requesting Text List".to_string())
                        }

                        messages::TextRequest::Text(n) => {
                            h2 = Line::from(format!("Requesting Text #{n}"));
                        }
                    }
                }
                messages::RequestType::MediaRequest(media_request) => {
                    rtype = Span::styled("Media", Style::new().bg(MEDIA_MSG));
                    match media_request {
                        messages::MediaRequest::MediaList => {
                            h2 = Line::from("Requesting Media List".to_string())
                        }
                        messages::MediaRequest::Media(n) => {
                            h2 = Line::from(format!("Requesting Media #{n}"));
                        }
                    }
                }
                messages::RequestType::ChatRequest(chat_request) => {
                    rtype = Span::styled("Chat", Style::new().bg(CHAT_MSG));
                    match chat_request {
                        messages::ChatRequest::ClientList => {
                            h2 = Line::from("Requesting Client List".to_string())
                        }
                        messages::ChatRequest::Register(n) => {
                            h2 = Line::from(format!("Requesting to register to chat #{n}"))
                        }
                        messages::ChatRequest::SendMessage { from, to, message } => {
                            h2 = Line::from(format!("Sending from #{from} to #{to} message: "));
                            opt = Line::from(message.as_str());
                        }
                    }
                }
                messages::RequestType::DiscoveryRequest(_) => {
                    rtype = Span::styled("Discovery", Style::new().bg(DISCOVERY_MSG));
                    {
                        h2 = Line::from("Sending Discovery.".to_string())
                    }
                }
            }
        }
        messages::MessageType::Response(response_type) => {
            mtype = Span::styled("Response", Style::new().bg(MESSAGE_RESPONSE_COLOR));
            match response_type {
                messages::ResponseType::TextResponse(text_response) => {
                    rtype = Span::styled("Text", Style::new().bg(TEXT_MSG));
                    match text_response {
                        messages::TextResponse::TextList(vec) => {
                            h2 = Line::from("Returning Text List:".to_string());
                            opt = Line::from(format!("{:?}", vec));
                        }
                        messages::TextResponse::Text(text) => {
                            h2 = Line::from("Returning TextResponse:".to_string());
                            opt = Line::from(text.as_str());
                        }
                        messages::TextResponse::NotFound => {
                            h2 = Line::from("TextResponse::NotFound".to_string());
                        }
                    }
                }
                messages::ResponseType::MediaResponse(media_response) => {
                    rtype = Span::styled("Media", Style::new().bg(MEDIA_MSG));
                    match media_response {
                        messages::MediaResponse::MediaList(vec) => {
                            h2 = Line::from("Returning Media List:".to_string());
                            opt = Line::from(format!("{:?}", vec));
                        }
                        messages::MediaResponse::Media(vec) => {
                            h2 = Line::from("Returning Media:".to_string());
                            opt = Line::from(format!("{:?}", vec));
                        }
                    }
                }
                messages::ResponseType::ChatResponse(chat_response) => {
                    rtype = Span::styled("Chat", Style::new().bg(CHAT_MSG));
                    match chat_response {
                        messages::ChatResponse::ClientList(vec) => {
                            h2 = Line::from("Returning  List:".to_string());
                            opt = Line::from(format!("{:?}", vec));
                        }
                        messages::ChatResponse::MessageFrom { from, message } => {
                            h2 = Line::from(format!("Response from #{from}, message: "));
                            opt = Line::from(format!("{:?}", message));
                        }
                        messages::ChatResponse::MessageSent => {
                            h2 = Line::from("Response: MessageSent".to_string());
                        }
                    }
                }
                messages::ResponseType::DiscoveryResponse(server_type) => {
                    rtype = Span::styled("Discovery", Style::new().bg(DISCOVERY_MSG));
                    match server_type {
                        messages::ServerType::ContentServer => {
                            h2 = Line::from(
                                "Discovery Response: server is of Content type".to_string(),
                            );
                        }
                        messages::ServerType::CommunicationServer => {
                            h2 = Line::from(
                                "Discovery Response: server is of Communication type".to_string(),
                            );
                        }
                    }
                }
            }
        }
        messages::MessageType::Error(error_type) => {
            mtype = Span::styled("Response", Style::new().bg(MESSAGE_ERROR_COLOR));
            match error_type {
                messages::ErrorType::Unsupported(request_type) => {
                    rtype = Span::styled("Unsupported", Style::new().bg(TEXT_MSG));
                    h2 = Line::from(format!("request type: {:?}", request_type));
                }
                messages::ErrorType::Unexpected(response_type) => {
                    rtype = Span::styled("Unexpected", Style::new().bg(TEXT_MSG));
                    h2 = Line::from(format!("response type: {:?}", response_type));
                }
            }
        }
    }
    let mut header = Line::default();
    header.push_span(rtype);
    header.push_span(mtype);

    res.push_line(header);
    res.push_line(h2);
    res.push_line(opt);
    Paragraph::new(res).wrap(Wrap { trim: true })
}

pub fn packet_table_row(packet: &Packet) -> Row {
    let sess_id: Span = Span::styled(format!("{}", packet.session_id), Style::new());

    let src: Span;
    let dest: Span;
    match &packet.pack_type {
        PacketType::FloodRequest(floodrequest) => {
            if floodrequest.path_trace.is_empty() {
                src = Span::styled("X".to_string(), Style::new());
                dest = Span::styled("X".to_string(), Style::new());
            } else {
                src = Span::styled(
                    format!("{}", floodrequest.path_trace.last().unwrap().0),
                    Style::new(),
                );
                // there really should not be any way to know the destination, using the routing
                // header is inconsistent
                //if let Some(dst) = packet.routing_header.current_hop() {
                //    dest = Span::styled(format!("{dst}"), Style::new());
                //} else {
                dest = Span::styled("X".to_string(), Style::new());
                //}
            }
        }
        _ => {
            let h = &packet.routing_header.hops;
            let hi = packet.routing_header.hop_index;
            if h.len() > 1 && hi < h.len() && hi > 1 {
                let s = h[hi - 1];
                let d = h[hi];
                src = Span::styled(format!("{}", s), Style::new());
                dest = Span::styled(format!("{}", d), Style::new());
            } else {
                src = Span::styled("X".to_string(), Style::new());
                dest = Span::styled("X".to_string(), Style::new());
            }
        }
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
