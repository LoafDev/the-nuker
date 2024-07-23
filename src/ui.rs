use crate::app::{App, CurrentScreen};
use ratatui::{
    prelude::{Layout, Frame, Direction, Constraint, Style, Text, Color, Span, Alignment, Line, Rect, Stylize},
    widgets::{Block, Borders, Paragraph, Clear, ListItem, List, HighlightSpacing}
};

pub fn ui(f: &mut Frame, app: &mut App) {
    //render 3 chunks
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(3),
    ]).split(f.size());

    //render title at upper chunk (chunks[0])
    let title_block = Block::default()
    .borders(Borders::BOTTOM)
    .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Viewing folders!",
        Style::default().fg(Color::Blue)
    )).block(title_block);

    f.render_widget(title, chunks[0]);

    //render mid chunk!
    draw_list(f, app);

    //render navigation help at lower chunk (chunks[2])
    let navigation_help_block = Block::default()
    .borders(Borders::TOP)
    .style(Style::default());

    let navi_text = Span::from(
        Span::styled(
            "Arrows to move, Enter to choose! If you don't want to, just do the Q, but if you wish to big D will do! Yes or no just choose!",
            Style::default().fg(Color::Yellow)
        )
    );
    
    let navigation = Paragraph::new(navi_text)
    .block(navigation_help_block)
    .centered();

    f.render_widget(navigation, chunks[2]);
 
    //render ask pop_up and path deleted messsage popup
    if let CurrentScreen::ConfirmPath = &app.current_screen {
        render_popup(f, "Path confirmation", &format!("Do you wish to delete {:?}? y/n", app.path));
    } else if let CurrentScreen::DeletedPath = &app.current_screen {
        render_popup(f, "Deleted path message", "Deleted path successfully! Press Enter to quit!");
    }
} 

pub fn draw_list(f: &mut Frame, app: &mut App) {
    //render list of folders and files at middle (chunks[1])
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(2),
        Constraint::Min(1),
    ]).split(f.size());

    let mut buf = Vec::<ListItem>::new();
    
    for files in app.file_list.iter() {
        buf.push(ListItem::new(Line::from(Span::styled(
            format!("{:?}", files), Style::default().fg(Color::Blue)
        ))));
    }

    let list = List::new(buf)
    .highlight_style(Style::default().fg(Color::LightYellow))
    .highlight_symbol(">")
    .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);
}

pub fn render_popup(f: &mut Frame, block_mess: &str, para_mess: &str) {
    let area = centered_rect(80, 80, f.size());

    let block = Block::new().borders(Borders::ALL).title(Span::styled(block_mess, Style::default().fg(Color::Green).bold())).title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(Span::styled(para_mess, Style::default().fg(Color::Red).bold()))
    .block(block)
    .centered();

    f.render_widget(Clear, area);
    f.render_widget(paragraph,  area);
}


pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage((100 - percent_y) /2),
        Constraint::Percentage(percent_y /2),
        Constraint::Percentage((100 - percent_y) /2)
    ]).split(r);

    Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage((100 - percent_x) /2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) /2)
    ]).split(popup_layout[1])[1]
}