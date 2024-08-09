use crate::app::{App, MenuOptions};
use itertools::Itertools;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem};
use ratatui::Frame;
use strum::VariantArray;

pub fn build_home_screen(f: &mut Frame, app: &App, area: Rect) {
    let list_items = MenuOptions::VARIANTS
        .iter()
        .map(|menu_item| {
            let style = if app.get_main_menu_selected_option() == *menu_item {
                Style::default().bg(Color::White).fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(menu_item.to_string(), style)))
        })
        .collect_vec();
    let list = List::new(list_items).block(Block::default());

    f.render_widget(list, area)
}
