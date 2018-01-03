#[macro_use]
extern crate error_chain;
extern crate rpassword;
extern crate term_painter;

mod errors {
    error_chain!{
        foreign_links {
            IoError(::std::io::Error);
        }
    }
}

use errors::*;
use std::io::{stdin, Cursor, BufRead, Write};
use std::fmt::Display;
use rpassword::{read_password, read_password_with_reader};
use term_painter::Color::*;
use term_painter::ToStyle;

pub type Answer = String;

pub enum Colour {
    Red,
    Green,
    Blue,
}

struct Colours {
    fg: term_painter::Color,
    bg: term_painter::Color,
}

#[derive(Default)]
struct State<'ask> {
    no_answer: Option<bool>,
    no_echo: Option<bool>,
    confirm: Option<bool>,
    default: Option<&'ask str>,
    prompt: Option<&'ask char>,
    bg_colour: Option<Colour>,
    fg_colour: Option<Colour>,
    validate: Option<&'ask Fn(Answer) -> bool>,
    redirect_in: Option<Cursor<&'ask [u8]>>,
    redirect_out: Option<&'ask mut Write>,
}

#[derive(Default)]
pub struct StateBuilder<'ask, T: Display + Default> {
    msg: T,
    state: State<'ask>,
}

impl<'ask, T: Display + Default> StateBuilder<'ask, T> {
    fn print_message(&mut self, colours: &Colours) -> Result<()> {
        let message = match self.state.prompt {
            Some(prompt) => format!("{}{}", self.msg, prompt),
            None => format!("{}", self.msg),
        };
        self.print(colours, message)
    }

    fn print<D: Display>(&mut self, colours: &Colours, msg: D) -> Result<()> {
        let output = format!("{}", colours.fg.bg(colours.bg).paint(msg));
        match self.state.redirect_out {
            Some(ref mut w) => w.write_all(output.as_bytes())?,
            None => println!("{}", output),
        };
        Ok(())
    }

    fn print_confirm(&mut self, colours: &Colours) -> Result<()> {
        self.print(colours, "Are you sure? Y/N")
    }

    fn read_no_echo(&mut self) -> Result<Answer> {
        self.check_colour()?;
        let response = match self.state.redirect_in {
            Some(ref mut cur) => read_password_with_reader(Some(cur)),
            _ => read_password(),
        };
        Ok(response.chain_err(|| "Unable to read input.")?)
    }

    fn check_no_echo(&mut self) -> Result<Answer> {
        match self.state.no_echo {
            Some(true) => self.read_no_echo(),
            _ => {
                self.check_colour()?;
                self.read()
            }
        }
    }

    fn get_fg_colour(&mut self) -> term_painter::Color {
        match self.state.fg_colour {
            Some(Colour::Red) => Red,
            Some(Colour::Blue) => Blue,
            Some(Colour::Green) => Green,
            _ => White,
        }
    }

    fn get_bg_colour(&mut self) -> term_painter::Color {
        match self.state.bg_colour {
            Some(Colour::Red) => Red,
            Some(Colour::Blue) => Blue,
            Some(Colour::Green) => Green,
            _ => Black,
        }
    }

    fn loop_confirm(&mut self, colours: &Colours) -> Result<()> {
        loop {
            self.print_message(colours)?;
            self.print_confirm(colours)?;
            match self.read() {
                Ok(input) => {
                    if input.to_string() == "y" || input.to_string() == "Y" {
                        break;
                    }
                }
                _ => (),
            };
        }
        Ok(())
    }

    fn check_confirm(&mut self, colours: &Colours) -> Result<()> {
        match self.state.confirm {
            Some(true) => self.loop_confirm(colours),
            _ => self.print_message(colours),
        }
    }

    fn check_colour(&mut self) -> Result<()> {
        let colours = Colours {
            fg: self.get_fg_colour(),
            bg: self.get_bg_colour(),
        };
        self.check_confirm(&colours)
    }

    fn read(&mut self) -> Result<Answer> {
        let mut buf = String::new();
        match self.state.redirect_in {
            Some(ref mut cur) => {
                cur.read_line(&mut buf).chain_err(|| "Unable to read from input.")?
            }
            None => stdin().read_line(&mut buf).chain_err(|| "Unable to read from STDIN.")?,
        };
        Ok(buf)
    }

    fn check_validation(&self, answer: Answer) -> Result<Answer> {
        if let Some(f) = self.state.validate {
            if f(answer.clone()) {
                Ok(answer)
            } else {
                Err("Response failed validation".into())
            }
        } else {
            Ok(answer)
        }
    }

    fn discard_answer(&self, answer: Answer) -> Result<Answer> {
        match self.state.no_answer {
            Some(true) => Ok(String::new()),
            _ => Ok(answer),
        }
    }

    pub fn ask(&mut self) -> Result<Answer> {
        let answer = self.check_no_echo()?;
        let validated_answer = self.check_validation(answer)?;
        self.discard_answer(validated_answer)
    }

    pub fn no_echo(&mut self) -> &'ask mut StateBuilder<T> {
        self.state.no_echo = Some(true);
        self
    }

    pub fn no_answer(&mut self) -> &'ask mut StateBuilder<T> {
        self.state.no_answer = Some(true);
        self
    }

    pub fn confirm(&mut self) -> &'ask mut StateBuilder<T> {
        self.state.confirm = Some(true);
        self
    }

    pub fn default(&mut self, msg: &'ask str) -> &'ask mut StateBuilder<T> {
        self.state.default = Some(msg);
        self
    }

    pub fn prompt(&mut self, prompt: &'ask char) -> &'ask mut StateBuilder<T> {
        self.state.prompt = Some(prompt);
        self
    }

    pub fn bg_colour(&mut self, colour: Colour) -> &'ask mut StateBuilder<T> {
        self.state.bg_colour = Some(colour);
        self
    }

    pub fn fg_colour(&mut self, colour: Colour) -> &'ask mut StateBuilder<T> {
        self.state.fg_colour = Some(colour);
        self
    }

    pub fn validate(&mut self, f: &'ask (Fn(Answer) -> bool)) -> &'ask mut StateBuilder<T> {
        self.state.validate = Some(f);
        self
    }

    pub fn redirect_in(&mut self, cur: Cursor<&'ask [u8]>) -> &'ask mut StateBuilder<T> {
        self.state.redirect_in = Some(cur);
        self
    }

    pub fn redirect_out<W: Write>(&mut self, w: &'ask mut W) -> &'ask mut StateBuilder<T> {
        self.state.redirect_out = Some(w);
        self
    }
}

pub fn input<'ask, T: Display + Default>(msg: T) -> StateBuilder<'ask, T> {
    StateBuilder { msg: msg, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use ::{input, Colour, Answer, Cursor};

    const MSG: &str = "A test message.";
    const DEFAULT_RESPONSE: &str = "A response.";
    const EMPTY_RESPONSE: &str = "";

    enum Message {
        TestMessage,
    }

    impl fmt::Display for Message {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Test Message")
        }
    }

    impl Default for Message {
        fn default() -> Message {
            Message::TestMessage
        }
    }

    fn mock_input() -> Cursor<&'static [u8]> {
        Cursor::new(&b"A response."[..])
    }

    fn mock_no_echo() -> Cursor<&'static [u8]> {
        Cursor::new(&b"A response.\r\n"[..])
    }

    fn mock_confirm() -> Cursor<&'static [u8]> {
        Cursor::new(&b"y"[..])
    }

    #[test]
    fn can_ask_a_question() {
        assert_eq!(input(MSG).redirect_in(mock_input()).ask().unwrap(),
                   DEFAULT_RESPONSE);
    }

    #[test]
    fn can_disable_echo() {
        assert_eq!(input(MSG).redirect_in(mock_no_echo()).no_echo().ask().unwrap(),
                   DEFAULT_RESPONSE);
    }

    #[test]
    fn can_disable_answer() {
        assert_eq!(input(MSG).redirect_in(mock_input()).no_answer().ask().unwrap(),
                   EMPTY_RESPONSE);
    }

    #[test]
    fn can_chain_operations() {
        assert_eq!(input(MSG).redirect_in(mock_input()).no_answer().ask().unwrap(),
                   EMPTY_RESPONSE);
    }

    #[test]
    fn can_set_prompt() {
        assert_eq!(input(MSG).redirect_in(mock_input()).prompt(&':').ask().unwrap(),
                   DEFAULT_RESPONSE)
    }

    #[test]
    fn can_redirect_output() {
        let mut sink = ::std::io::sink();
        assert_eq!(input(MSG).redirect_out(&mut sink).redirect_in(mock_input()).ask().unwrap(),
                   DEFAULT_RESPONSE);
    }

    #[test]
    fn can_set_fg_colour() {
        assert_eq!(input(MSG).redirect_in(mock_input()).fg_colour(Colour::Red).ask().unwrap(),
                   DEFAULT_RESPONSE);
    }

    #[test]
    fn can_set_bg_colour() {
        assert_eq!(input(MSG).redirect_in(mock_input()).bg_colour(Colour::Red).ask().unwrap(),
                   DEFAULT_RESPONSE);
    }

    #[test]
    fn can_ask_for_confirmation() {
        assert_eq!(input(MSG).redirect_in(mock_confirm()).confirm().no_answer().ask().unwrap(),
                   EMPTY_RESPONSE);
    }

    #[test]
    fn can_validate_answer() {
        let valid = |a: Answer| -> bool { if a == DEFAULT_RESPONSE { true } else { false } };
        assert_eq!(input(MSG).redirect_in(mock_input()).validate(&valid).ask().unwrap(),
                   DEFAULT_RESPONSE);
    }

    #[test]
    fn can_accept_any_type_implementing_display_and_default() {
        let message = Message::TestMessage;
        assert_eq!(input(message).redirect_in(mock_input()).ask().unwrap(),
                   DEFAULT_RESPONSE);
    }
}
