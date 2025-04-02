use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::Text,
    widgets::{StatefulWidget, Widget},
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage};

use crate::{app_context::AppContext, tg::message_entry::MessageEntry};

const MAX_IMAGE_HEIGHT: u16 = 15;

struct ChatEntry<'a> {
    text: Option<Text<'a>>,
    images: Vec<StatefulProtocol>,
}

struct ChatState {
    offset: usize,
    selected: Option<usize>,
}

impl ChatState {
    fn new() -> Self {
        Self {
            offset: 0,
            selected: None,
        }
    }
}

struct Chat<'a> {
    app_context: &'a AppContext,
    items: Vec<ChatEntry<'a>>,
}

impl<'a> Chat<'a> {
    fn new(app_context: &'a AppContext, items: &'a [MessageEntry], area: &Rect) -> Self {
        // TODO Proper error handling
        let pick = Picker::from_query_stdio().unwrap();
        let mut is_unread_outbox = true;
        let mut is_unread_inbox = true; // Seems to be unused
        let wrap_width = (area.width / 2) as i32;
        let items = items
            .iter()
            .map(|entry| {
                let (myself, name_style, content_style, alignment) =
                    if entry.sender_id() == app_context.tg_context().me() {
                        if entry.id() == app_context.tg_context().last_read_outbox_message_id() {
                            is_unread_outbox = false;
                        }
                        (
                            true,
                            app_context.style_chat_message_myself_name(),
                            app_context.style_chat_message_myself_content(),
                            Alignment::Right,
                        )
                    } else {
                        if entry.id() == app_context.tg_context().last_read_inbox_message_id() {
                            is_unread_inbox = false;
                        }
                        (
                            false,
                            app_context.style_chat_message_other_name(),
                            app_context.style_chat_message_other_content(),
                            Alignment::Left,
                        )
                    };
                let text = Some(
                    entry
                        .get_text_styled(
                            myself,
                            &app_context,
                            is_unread_outbox,
                            name_style,
                            content_style,
                            wrap_width,
                        )
                        .alignment(alignment),
                );
                ChatEntry {
                    text,
                    images: entry
                        .photos()
                        .iter()
                        .map(|p| {
                            // TODO Proper error handling
                            let im = image::load_from_memory(p.as_bytes()).unwrap();
                            pick.new_resize_protocol(im)
                        })
                        .collect(),
                }
            })
            .collect();
        Chat { app_context, items }
    }
}

impl StatefulWidget for Chat<'_> {
    type State = ChatState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let end = area.y;
        let mut current = area.y + area.height - 1;
        for item in self.items.iter_mut() {
            if current < end {
                return;
            }

            if let Some(t) = item.text.as_ref() {
                if current - (t.height() as u16) < end {
                    return;
                }

                let text = t.clone();
                let t_h = text.height() as u16;
                text.render(Rect::new(area.x, current - (t_h - 1), area.width, t_h), buf);
                current -= t_h;
            }

            for image in item.images.iter_mut() {
                if current < end {
                    return;
                }

                if current - MAX_IMAGE_HEIGHT < end {
                    return;
                }

                let to_draw = StatefulImage::new().resize(Resize::Scale(None));
                to_draw.render(
                    Rect::new(
                        area.x,
                        current - (MAX_IMAGE_HEIGHT - 1),
                        area.width,
                        MAX_IMAGE_HEIGHT,
                    ),
                    buf,
                    image,
                );
                current -= MAX_IMAGE_HEIGHT;
            }
        }
    }
}
