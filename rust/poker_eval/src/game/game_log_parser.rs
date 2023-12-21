use log::trace;
use regex::Regex;

use crate::{PokerError, InitialPlayerState, ChipType};

pub struct GameLogParser {
    pub section_name_regex: Regex,
    pub player_stack_regex: Regex,
    pub blinds_regex: Regex,
}

impl GameLogParser {
    pub fn new() -> Self {

        Self {
            section_name_regex: Regex::new(
                r#"(?x) # Enable verbose mode            
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    \*+\s*                  # Matches one or more '*' followed by any whitespace
    (?P<section_name>.+?)   # Lazily captures one or more characters as the section name
    \s*\*+                  # Matches any trailing whitespace followed by one or more '*'
"#,
            )
            .unwrap(),

            player_stack_regex: Regex::new(
                r#"(?x) # Enable verbose mode
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    (?P<player_id>[\w\ ]+)  # Capture player id
    \ *                     # Match any trailing spaces
    -                       # Match a dash
    \ *                     # Match any trailing spaces
    (?P<stack>[\d\.\,]+)    # Capture stack
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    "#,
            )
            .unwrap(), 
            blinds_regex: Regex::new(
                r#"(?x) # Enable verbose mode
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    (?P<p1>[\w\ ]+)  # Capture player id
    \ *                     # Match any trailing spaces
    -                       # Match a dash
    \ *                     # Match any trailing spaces
    (?P<sb>[\d\.\,]+)    # Capture sb
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    \s*
    (?P<p2>[\w\ ]+)  # Capture player id
    \ *                     # Match any trailing spaces
    -                       # Match a dash
    \ *                     # Match any trailing spaces
    (?P<bb>[\d\.\,]+)    # Capture bb
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    "#,
            )
            .unwrap(),
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

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        Ok((section_name, remaining_str))
    }

    //will either return next player, or none if we are at the end of the section
    pub fn parse_players<'a>(
        &'a self,
        s: &'a str,
    ) -> Result< (Vec<InitialPlayerState>, &str), PokerError> {

        let mut ret = Vec::new();
        let mut remaining_str = s;
        for _ in 0..15 {
            let caps = self.player_stack_regex.captures(remaining_str);

            if caps.is_none() {
                break;
            }
            let caps = caps.unwrap();

            trace!("Total match {}", caps.get(0).unwrap().as_str());

            let player_id = caps.name("player_id").unwrap().as_str();
            trace!("Player id: {}", player_id);

            let stack_str = caps.name("stack").unwrap().as_str();
            trace!("Stack: {}", stack_str);

            let stack: ChipType = stack_str
                .replace(",", "")
                .parse()
                .map_err(|_| PokerError::from_string(format!("Could not parse stack {}", stack_str)))?;

            ret.push(InitialPlayerState{
                player_name: player_id.to_string(),
                stack,
                cards: None,
                position: ret.len().try_into()?
            });

            let match_end = caps.get(0).unwrap().end();
            remaining_str = &remaining_str[match_end..];

            trace!(
                "Remaining string len: {} start {}",
                remaining_str.len(),
                &remaining_str[0..10]
            );
        }

        Ok((ret, remaining_str))
    }

    pub fn parse_blinds<'a>(&'a self, 
    players: &Vec<InitialPlayerState>,
    s: &'a str) -> Result<( ChipType, ChipType, &str), PokerError> {
        let caps = self
            .blinds_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected blinds in {}"
            , &s[0..100])))?;

        let p1_name = caps.name("p1").unwrap().as_str();
        let p2_name = caps.name("p2").unwrap().as_str();
        let sb_str = caps.name("sb").unwrap().as_str();
        let bb_str = caps.name("bb").unwrap().as_str();

        trace!("p1: {}", p1_name);
        trace!("p2: {}", p2_name);
        trace!("sb: {}", sb_str);
        trace!("bb: {}", bb_str);

        if players.len() < 2 {
            return Err(PokerError::from_string(format!(
                "Expected at least 2 players, got {}"
            , players.len())));
        }

        if p1_name != players[0].player_name {
            return Err(PokerError::from_string(format!(
                "Expected small blind to be {} not {}",
                players[0].player_name, p1_name)));
        }

        if p2_name != players[1].player_name {
            return Err(PokerError::from_string(format!(
                "Expected big blind to be {} not {}",
                players[1].player_name, p2_name)));
        }

        let sb: ChipType = sb_str
            .replace(",", "")
            .parse()
            .map_err(|_| PokerError::from_string(format!("Could not parse sb {}", sb_str)))?;
        let bb: ChipType = bb_str
            .replace(",", "")
            .parse()
            .map_err(|_| PokerError::from_string(format!("Could not parse bb {}", bb_str)))?;

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        Ok(( sb, bb, remaining_str))
    }
}
