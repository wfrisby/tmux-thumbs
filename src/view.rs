use std::char;
use std::default::Default;
use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;
use super::*;

pub struct View<'a> {
  state: &'a mut state::State<'a>,
  skip: Option<usize>,
  reverse: bool,
  unique: bool,
  position: &'a str,
  select_foreground_color: Color,
  foreground_color: Color,
  background_color: Color,
  hint_background_color: Color,
  hint_foreground_color: Color
}

impl<'a> View<'a> {
  pub fn new(state: &'a mut state::State<'a>, reverse: bool, unique: bool, position: &'a str, select_foreground_color: Color, foreground_color: Color, background_color: Color, hint_foreground_color: Color, hint_background_color: Color) -> View<'a> {
    View{
      state: state,
      skip: None,
      reverse: reverse,
      unique: unique,
      position: position,
      select_foreground_color: select_foreground_color,
      foreground_color: foreground_color,
      background_color: background_color,
      hint_foreground_color: hint_foreground_color,
      hint_background_color: hint_background_color
    }
  }

  fn init(&mut self, max: usize) -> Option<usize> {
    if self.reverse {
      Some(0)
    } else {
      Some(max)
    }
  }

  pub fn prev(&mut self, max: usize) {
    self.skip = if let Some(value) = self.skip {
      if value > 0 {
        Some(value - 1)
      } else {
        Some(value)
      }
    } else {
      self.init(max)
    };
  }

  pub fn next(&mut self, max: usize) {
    self.skip = if let Some(value) = self.skip {
      if value < max {
        Some(value + 1)
      } else {
        Some(value)
      }
    } else {
      self.init(max)
    };
  }

  pub fn present(&mut self) -> Option<(String, bool)> {
    let mut rustbox = match RustBox::init(Default::default()) {
      Result::Ok(v) => v,
      Result::Err(e) => panic!("{}", e),
    };

    rustbox.set_output_mode(OutputMode::EightBit);

    for (index, line) in self.state.lines.iter().enumerate() {
      let clean = line.trim_end_matches(|c: char| c.is_whitespace());

      if clean.len() > 0 {
        let formatted = format!("{}\n", line).to_string();
        rustbox.print(0, index, rustbox::RB_NORMAL, Color::White, Color::Black, formatted.as_str());
      }
    }

    let mut typed_hint: String = "".to_owned();
    let matches = self.state.matches(self.reverse, self.unique);
    let longest_hint = matches.iter().filter_map(|m| m.hint.clone()).max_by(|x, y| x.len().cmp(&y.len())).unwrap().clone();
    let mut selected = None;

    loop {
      match matches.iter().enumerate().find(|&h| h.0 == self.skip.unwrap_or(5000)) {
        Some(hm) => {
          selected = Some(hm.1);
        }
        _ => {}
      }

      for mat in matches.iter() {
        let selected_color = if selected == Some(mat) {
          self.select_foreground_color
        } else {
          self.foreground_color
        };

        // Find long utf sequences and extract it from mat.x
        let line = &self.state.lines[mat.y as usize];
        let prefix = &line[0..mat.x as usize];
        let extra = prefix.len() - prefix.chars().count();
        let offset = (mat.x as usize) - extra;

        rustbox.print(offset, mat.y as usize, rustbox::RB_NORMAL, selected_color, self.background_color, mat.text);

        if let Some(ref hint) = mat.hint {
          let extra_position = if self.position == "left" { 0 } else { mat.text.len() - mat.hint.clone().unwrap().len() };

          rustbox.print(offset + extra_position, mat.y as usize, rustbox::RB_BOLD, self.hint_foreground_color, self.hint_background_color, hint.as_str());
        }
      }

      rustbox.present();

      let max = matches.len();
      self.skip = self.init(max);

      match rustbox.poll_event(false) {
        Ok(rustbox::Event::KeyEvent(key)) => {
          match key {
            Key::Esc => { break; }
            Key::Enter => {
              match matches.iter().enumerate().find(|&h| h.0 == self.skip.unwrap_or(5000)) {
                Some(hm) => {
                  return Some((hm.1.text.to_string(), false))
                }
                _ => panic!("Match not found?"),
              }
            }
            Key::Up => { self.prev(max); }
            Key::Down => { self.next(max); }
            Key::Left => { self.prev(max); }
            Key::Right => { self.next(max); }
            Key::Char(ch) => {
              let key = ch.to_string();
              let lower_key = key.to_lowercase();

              typed_hint.push_str(lower_key.as_str());

              match matches.iter().find(|mat| mat.hint == Some(typed_hint.clone())) {
                Some(mat) => {
                  return Some((mat.text.to_string(), key != lower_key))
                },
                None => {
                  if typed_hint.len() >= longest_hint.len() {
                    break;
                  }
                }
              }
            }
            _ => {}
          }
        }
        Err(e) => panic!("{}", e),
        _ => { }
      }
    }

    None
  }
}
