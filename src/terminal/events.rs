use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
}

impl From<Event> for TerminalEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => TerminalEvent::Key(key),
            Event::Mouse(mouse) => TerminalEvent::Mouse(mouse),
            Event::Resize(width, height) => TerminalEvent::Resize(width, height),
            Event::FocusGained => TerminalEvent::FocusGained,
            Event::FocusLost => TerminalEvent::FocusLost,
            Event::Paste(text) => TerminalEvent::Paste(text),
        }
    }
}

impl TerminalEvent {
    pub fn is_key(&self) -> bool {
        matches!(self, TerminalEvent::Key(_))
    }

    pub fn is_mouse(&self) -> bool {
        matches!(self, TerminalEvent::Mouse(_))
    }

    pub fn is_resize(&self) -> bool {
        matches!(self, TerminalEvent::Resize(_, _))
    }

    pub fn is_ctrl_c(&self) -> bool {
        if let TerminalEvent::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        }) = self
        {
            modifiers.contains(KeyModifiers::CONTROL)
        } else {
            false
        }
    }

    pub fn is_ctrl_d(&self) -> bool {
        if let TerminalEvent::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers,
            ..
        }) = self
        {
            modifiers.contains(KeyModifiers::CONTROL)
        } else {
            false
        }
    }

    pub fn is_enter(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })
        )
    }

    pub fn is_tab(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Tab,
                ..
            })
        )
    }

    pub fn is_backspace(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            })
        )
    }

    pub fn is_delete(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Delete,
                ..
            })
        )
    }

    pub fn is_arrow_up(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Up,
                ..
            })
        )
    }

    pub fn is_arrow_down(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            })
        )
    }

    pub fn is_arrow_left(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            })
        )
    }

    pub fn is_arrow_right(&self) -> bool {
        matches!(
            self,
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            })
        )
    }

    pub fn get_char(&self) -> Option<char> {
        if let TerminalEvent::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        }) = self
        {
            Some(*c)
        } else {
            None
        }
    }

    pub fn has_ctrl(&self) -> bool {
        if let TerminalEvent::Key(KeyEvent { modifiers, .. }) = self {
            modifiers.contains(KeyModifiers::CONTROL)
        } else {
            false
        }
    }

    pub fn has_alt(&self) -> bool {
        if let TerminalEvent::Key(KeyEvent { modifiers, .. }) = self {
            modifiers.contains(KeyModifiers::ALT)
        } else {
            false
        }
    }

    pub fn has_shift(&self) -> bool {
        if let TerminalEvent::Key(KeyEvent { modifiers, .. }) = self {
            modifiers.contains(KeyModifiers::SHIFT)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_c_detection() {
        let event = TerminalEvent::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ));
        assert!(event.is_ctrl_c());
    }

    #[test]
    fn test_enter_detection() {
        let event = TerminalEvent::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        ));
        assert!(event.is_enter());
    }

    #[test]
    fn test_char_extraction() {
        let event = TerminalEvent::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        ));
        assert_eq!(event.get_char(), Some('a'));
    }

    #[test]
    fn test_modifier_detection() {
        let event = TerminalEvent::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));
        assert!(event.has_ctrl());
        assert!(event.has_shift());
        assert!(!event.has_alt());
    }
}