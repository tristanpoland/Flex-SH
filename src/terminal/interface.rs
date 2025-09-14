use super::colors::ColorScheme;
use anyhow::Result;
use crossterm::{
    cursor::{self, MoveTo},
    execute,
    style::{Print, ResetColor},
    terminal::{self, Clear, ClearType},
};
use std::io;

pub struct TerminalInterface {
    color_scheme: ColorScheme,
    width: u16,
    height: u16,
}

impl TerminalInterface {
    pub fn new(color_scheme: ColorScheme) -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            color_scheme,
            width,
            height,
        })
    }

    pub fn update_size(&mut self) -> Result<()> {
        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn clear_screen(&self) -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        Ok(())
    }

    pub fn clear_line(&self) -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn move_cursor(&self, x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), MoveTo(x, y))?;
        Ok(())
    }

    pub fn hide_cursor(&self) -> Result<()> {
        execute!(io::stdout(), cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor(&self) -> Result<()> {
        execute!(io::stdout(), cursor::Show)?;
        Ok(())
    }

    pub fn print_colored(&self, text: &str, _color_key: &str) -> Result<()> {
        // Simplified color support for now
        execute!(io::stdout(), Print(text))?;
        Ok(())
    }

    pub fn print_status_line(&self, status: &str) -> Result<()> {
        let y = self.height - 1;
        execute!(
            io::stdout(),
            MoveTo(0, y),
            Clear(ClearType::CurrentLine),
            Print(format!("{:width$}", status, width = self.width as usize))
        )?;
        Ok(())
    }

    pub fn print_completion_menu(&self, completions: &[String], selected: usize) -> Result<()> {
        let start_y = self.height.saturating_sub(completions.len() as u16 + 1);

        for (i, completion) in completions.iter().enumerate() {
            let y = start_y + i as u16;
            execute!(io::stdout(), MoveTo(0, y), Clear(ClearType::CurrentLine))?;

            if i == selected {
                execute!(io::stdout(), Print(format!("> {}", completion)))?;
            } else {
                execute!(io::stdout(), Print(format!("  {}", completion)))?;
            }
            execute!(io::stdout(), ResetColor)?;
        }
        Ok(())
    }

    pub fn get_dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    pub fn draw_border(&self, x: u16, y: u16, width: u16, height: u16, title: Option<&str>) -> Result<()> {
        let horizontal = "─".repeat((width - 2) as usize);
        let top_border = format!("┌{}┐", horizontal);
        let bottom_border = format!("└{}┘", horizontal);

        execute!(io::stdout(), MoveTo(x, y), Print(&top_border))?;

        for i in 1..height - 1 {
            execute!(
                io::stdout(),
                MoveTo(x, y + i),
                Print("│"),
                MoveTo(x + width - 1, y + i),
                Print("│")
            )?;
        }

        execute!(io::stdout(), MoveTo(x, y + height - 1), Print(&bottom_border))?;

        if let Some(title) = title {
            let title_x = x + (width - title.len() as u16) / 2;
            execute!(
                io::stdout(),
                MoveTo(title_x - 1, y),
                Print(format!(" {} ", title))
            )?;
        }

        Ok(())
    }

    pub fn draw_progress_bar(&self, x: u16, y: u16, width: u16, progress: f32, label: Option<&str>) -> Result<()> {
        let filled_width = ((width as f32 * progress).round() as u16).min(width);
        let empty_width = width - filled_width;

        execute!(io::stdout(), MoveTo(x, y))?;
        execute!(io::stdout(), Print("█".repeat(filled_width as usize)))?;
        execute!(io::stdout(), Print("░".repeat(empty_width as usize)))?;

        if let Some(label) = label {
            let percentage = (progress * 100.0).round() as u8;
            let text = format!("{} ({}%)", label, percentage);
            let text_x = x + (width - text.len() as u16) / 2;
            execute!(io::stdout(), MoveTo(text_x, y + 1), Print(&text))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_interface_creation() {
        let scheme = ColorScheme::default_scheme();
        let interface = TerminalInterface::new(scheme);
        assert!(interface.is_ok());
    }

    #[test]
    fn test_dimensions() {
        let scheme = ColorScheme::default_scheme();
        if let Ok(interface) = TerminalInterface::new(scheme) {
            let (width, height) = interface.get_dimensions();
            assert!(width > 0);
            assert!(height > 0);
        }
    }
}