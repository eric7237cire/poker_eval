use log::trace;
use regex::Regex;

use crate::PokerError;

pub struct GameLogParser {
    pub section_name_regex: Regex,
    pub player_stack_regex: Regex,
}

impl GameLogParser {
    pub fn new() -> Self {
        Self {
            section_name_regex: Regex::new(r#"(?x)    # Enable verbose mode            
    ^\s*                   # Asserts the start of the string and matches any leading whitespace
    \*+\s*                 # Matches one or more '*' followed by any whitespace
    (?P<section_name>.+?)  # Lazily captures one or more characters as the section name
    \s*\*+                # Matches any trailing whitespace followed by one or more '*'
"#).unwrap(),

            player_stack_regex: Regex::new(
                r#"(?x)  # Enable verbose mode
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    (?P<player_id>[\w\ ]+)  # Capture player id
    \ *                     # Match any trailing spaces
    -                       # Match a dash
    \ *                     # Match any trailing spaces
    (?P<stack>[\d\.\,]+)    # Capture stack
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    "#).unwrap(), //(\r\n|\r|\n|$)        # Match a newline (CR+LF, CR, LF) or end of string

        }
    }

    // Returns the section name and the remaining string
    pub fn parse_section_name<'a>(&'a self, s: &'a str) -> Result<(&str, &str), PokerError> {
        let caps = self
            .section_name_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected section *** section name ***"
            )))?;

        let section_name = caps.name("section_name").unwrap().as_str();
        trace!("Section name: {}", section_name);

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!("Remaining string: {}", remaining_str);

        Ok((section_name, remaining_str))
    }

    //will either return next player, or none if we are at the end of the section
    pub fn parse_player_name_stack<'a>(
        &'a self,
        s: &'a str,
    ) -> Result<(Option<(&str, usize)>, &str), PokerError> {
        let caps = self.player_stack_regex.captures(s);

        // if there are no more players we should have a new section
        if caps.is_none() {
            let (section_name, remaining_str) = self.parse_section_name(s)?;
            return Ok((None, s));
        }

        let caps = caps.unwrap();

        trace!("Total match {}", caps.get(0).unwrap().as_str());

        let player_id = caps.name("player_id").unwrap().as_str();
        trace!("Player id: {}", player_id);

        let stack = caps.name("stack").unwrap().as_str();
        trace!("Stack: {}", stack);

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!("Remaining string len: {} start {}", 
            remaining_str.len(),
            &remaining_str[0..10]);

        let opt = Some((player_id, 37));
        Ok((opt, remaining_str))
    }
}
