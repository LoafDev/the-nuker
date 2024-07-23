use std::io::Result;
mod app; mod ui;
use crate::{
    app::{App, CurrentScreen, iter_file},
    ui::ui
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{backend::{CrosstermBackend, Backend}, Terminal};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new()?;
    let res = run_app(&mut app, &mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        println!("{:?}",e);
    }
    Ok(())
}

fn run_app<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(k) = event::read()? {
            if k.kind == event::KeyEventKind::Release {
                continue;
            }

            match app.current_screen {
                CurrentScreen::ChoosePath => match k.code {
                    KeyCode::Up => {
                        app.list_state.select_previous();
                    }

                    KeyCode::Down => {
                        app.list_state.select_next();
                    }

                    KeyCode::Left => {
                        app.list_state.select_first();
                    }

                    KeyCode::Right => {
                        app.list_state.select_last();
                    }

                    KeyCode::Char('Q') => {
                        return Ok(());
                    }

                    KeyCode::Enter => {
                        if let Some(i) = app.list_state.selected() {
                            if app.file_list[i].is_dir() {
                                app.current_working_directory = app.file_list[i].to_owned();
                                app.file_list = iter_file(app.current_working_directory.to_owned()).expect("\x1b[1m\x1b[33mError:\x1b[0m Failed to parse file!");
                            }
                        }
                    }

                    KeyCode::Backspace => {
                        if let Some(p) = app.current_working_directory.parent() {
                            app.current_working_directory = p.to_path_buf();
                        }
                        app.file_list = iter_file(app.current_working_directory.to_owned()).expect("\x1b[1m\x1b[33mError:\x1b[0m Failed to parse file!");
                    }

                    KeyCode::Char('D') => {
                        if let Some(i) = app.list_state.selected() {
                            app.path = app.file_list[i].to_owned();
                            app.current_screen = CurrentScreen::ConfirmPath;
                        }
                    }
                    _ => {}
                }

                CurrentScreen::ConfirmPath => match k.code {
                    KeyCode::Char('y') => {
                        if let Some(i) = app.list_state.selected() {
                            app.file_list.remove(i);
                        }
                        app.clean();
                        app.current_screen = CurrentScreen::DeletedPath;
                    }

                    KeyCode::Char('n') => {
                        app.current_screen = CurrentScreen::ChoosePath;
                    }
                    _ => {}
                }

                CurrentScreen::DeletedPath => match k.code {
                    KeyCode::Enter => {
                        app.current_screen = CurrentScreen::ChoosePath;
                    }

                    KeyCode::Char('Q') => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}
