use crate::{HoleCards, CardVec};
use log::trace;
use regex::Regex;

use crate::{Card, ChipType, InitialPlayerState, PlayerAction, PokerError, Round};

use super::action;

pub struct GameLogParser {
    pub section_name_regex: Regex,
    pub player_stack_regex: Regex,
    pub blinds_regex: Regex,
    pub action_regex: Regex,
    pub cards_regex: Regex,
    
    pub trailing_comment_regex: Regex,

    pub player_id_regex: Regex,
    pub chip_amount_regex: Regex,

    pub get_word: Regex,
}

const ACTION_LIMIT: usize = 100;

impl GameLogParser {
    pub fn new() -> Self {
        Self {
player_id_regex: Regex::new(r#"(?x) # Enable verbose mode
^\s*                    # Asserts the start of the string and matches any leading whitespace
\b{start-half}          # Non consuming word boundary
(?P<player_id>[\w\ ]+)  # Capture player id
\b{end-half}            # Non consuming word boundary
"#).unwrap(),
chip_amount_regex: Regex::new(r#"(?x) # Enable verbose mode
^\s*                    # Asserts the start of the string and matches any leading whitespace
(?P<amount>[\d\.\,]+)   # Capture amount
\b{end-half}            # Non consuming word boundary
"#).unwrap(),
            trailing_comment_regex: Regex::new(
                r#"(?x) # Enable verbose mode
                (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
                "#).unwrap(),
             

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
    \b(?P<player_id>[\w\ ]+)\b  # Capture player id
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
    \b(?P<p1>[\w\ ]+)\b  # Capture player id
    \ *                     # Match any trailing spaces
    -                       # Match a dash
    \ *                     # Match any trailing spaces
    (?P<sb>[\d\.\,]+)    # Capture sb
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    \s*
    \b(?P<p2>[\w\ ]+)\b  # Capture player id with non consuming word boundaries
    \ *                     # Match any trailing spaces
    -                       # Match a dash
    \ *                     # Match any trailing spaces
    (?P<bb>[\d\.\,]+)    # Capture bb
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    "#,
            )
            .unwrap(),
            action_regex: Regex::new(
                r#"(?x) # Enable verbose mode
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    \b(?P<player_id>[\w\ ]+)\b{end-half}  # Capture player id
    \ *                     # Match any trailing spaces
    (?P<action>checks|bets|folds|raises|calls)    # Capture action
    \ *
    (?P<amount>[\d\.\,]+)?    # Capture optional amount
    (?:\s*\#[^\n\r]*)?      # Non-capturing group for # and anything after it, greedy
    "#,
            )
            .unwrap(),
            cards_regex: Regex::new(
                r#"(?x) # Enable verbose mode
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    (                       # start of seq
        (?:
        [\dakqjtAKQJT]      # value
        [cshdCSHD]          # suit 
        [\ ,]*              # can have spaces or commas between cards
        )+                  # one or more cards
    )                       # end capture group
    "#,
            )
            .unwrap(),
            get_word: Regex::new(
                r#"(?x) # Enable verbose mode
                ^\s*                    # Asserts the start of the string and matches any leading whitespace
                \b{start-half}          # Non consuming word boundary
                ([\w]+-)  # A word, or seperator
                \b{end-half}            # Non consuming word boundary
                "#).unwrap(),

        }
    }

    //returns player_id and remaining string
    //if players is passed, will make sure player exists
    pub fn parse_player_id<'a>(&'a self, s: &mut &'a str, players: Option<&Vec<InitialPlayerState>>,) -> Result<&str, PokerError> {
        let caps = self
            .player_id_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected player id *** player id ***"
            )))?;

        let player_id = caps.name("player_id").unwrap().as_str();
        trace!("Player id: {}", player_id);

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        if let Some(players) = players {
            if !players.iter().any(|p| p.player_name == player_id) {
                return Err(PokerError::from_string(format!(
                    "Expected player id [{}] to be in players",
                    player_id
                )));
            }
        }

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        *s = remaining_str;

        Ok(player_id)
    }

    pub fn parse_word<'a>(&'a self, s: &mut &'a str) -> Result<&str, PokerError> {
        let caps = self
            .player_amt_separator_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected player amount separator"
            )))?;

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        *s = remaining_str;

        let word = caps.get(1).ok_or(PokerError::from_string(format!(
            "Expected word"
        )))?.as_str();
        Ok(word)
    }

    pub fn parse_chip_amount<'a>(&'a self, s: &mut &'a str) -> Result<ChipType, PokerError> {
        let caps = self
            .chip_amount_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected chip amount"
            )))?;

        let amount_str = caps.name("amount").unwrap().as_str();
        trace!("Amount: {}", amount_str);

        let amount: ChipType = amount_str
            .replace(",", "")
            .parse()
            .map_err(|_| PokerError::from_string(format!("Could not parse amount {}", amount_str)))?;

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        *s = remaining_str;

        Ok(amount)
    }
    // Returns the section name and the remaining string
    pub fn parse_section_name<'a>(
        &'a self,
        s: &mut &'a str,
        expected: Option<&str>,
    ) -> Result<&str, PokerError> {
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

        if let Some(expected) = expected {
            if section_name != expected {
                return Err(PokerError::from_string(format!(
                    "Expected section [{}], got [{}]",
                    expected, section_name
                )));
            }
        }

        *s = remaining_str;

        Ok(section_name)
    }

    //will either return next player, or none if we are at the end of the section
    pub fn parse_players<'a>(
        &'a self,
        s: &'a str,
    ) -> Result<(Vec<InitialPlayerState>, &str), PokerError> {
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
            trace!("Player id: [{}]", player_id);

            let stack_str = caps.name("stack").unwrap().as_str();
            trace!("Stack: {}", stack_str);

            let stack: ChipType = stack_str.replace(",", "").parse().map_err(|_| {
                PokerError::from_string(format!("Could not parse stack {}", stack_str))
            })?;

            
            let match_end = caps.get(0).unwrap().end();
            remaining_str = &remaining_str[match_end..];

            //Parse optional cards
            let cards_result = self.parse_cards(remaining_str);

            let mut cards: Option<HoleCards> = None;

            if let Ok( (vec_cards, new_remaining_str)) = cards_result {
                
                remaining_str = new_remaining_str;
                
                if vec_cards.len() != 2 {
                    return Err(PokerError::from_string(format!(
                        "Expected 2 cards, got {}",
                        vec_cards.len()
                    )));
                }
                cards = Some(HoleCards::new(vec_cards[0], vec_cards[1])?);
                
            }

            ret.push(InitialPlayerState {
                player_name: player_id.to_string(),
                stack,
                cards,
                position: ret.len().try_into()?,
            });


            trace!(
                "Remaining string len: {} start {}",
                remaining_str.len(),
                &remaining_str[0..10]
            );
        }

        Ok((ret, remaining_str))
    }

    pub fn parse_blinds<'a>(
        &'a self,
        players: &Vec<InitialPlayerState>,
        s: &'a str,
    ) -> Result<(ChipType, ChipType, &str), PokerError> {
        let caps = self
            .blinds_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected blinds in {}",
                &s[0..100]
            )))?;

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
                "Expected at least 2 players, got {}",
                players.len()
            )));
        }

        if p1_name != players[0].player_name {
            return Err(PokerError::from_string(format!(
                "Expected small blind to be [{}] not [{}]",
                players[0].player_name, p1_name
            )));
        }

        if p2_name != players[1].player_name {
            return Err(PokerError::from_string(format!(
                "Expected big blind to be [{}] not [{}]",
                players[1].player_name, p2_name
            )));
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

        Ok((sb, bb, remaining_str))
    }

    pub fn parse_round_actions<'a>(
        &'a self,
        players: &Vec<InitialPlayerState>,
        round: Round,
        s: &'a str,
    ) -> Result<(Vec<PlayerAction>, &str), PokerError> {
        let mut ret = Vec::new();
        let mut remaining_str = s;
        for _ in 0..ACTION_LIMIT {
            let caps = self.action_regex.captures(remaining_str);

            if caps.is_none() {
                break;
            }
            let caps = caps.unwrap();

            trace!("Total match {}", caps.get(0).unwrap().as_str());

            let player_id = caps.name("player_id").unwrap().as_str();
            trace!("Player id: {}", player_id);

            let action_str = caps.name("action").unwrap().as_str();
            trace!("Action: {}", action_str);

            let amount_str = caps.name("amount").map(|m| m.as_str());

            let amount: Option<Result<ChipType, PokerError>> = amount_str.map(|s| {
                s.replace(",", "")
                    .parse()
                    .map_err(|_| PokerError::from_string(format!("Could not parse amount {}", s)))
            });

            //lookup index of player or return error if we don't find it
            let player_index = players
                .iter()
                .position(|p| p.player_name == player_id)
                .ok_or(PokerError::from_string(format!(
                    "Could not find player [{}] in round action",
                    player_id
                )))?;

            let action = match action_str {
                "checks" => action::ActionEnum::Check,
                "bets" => action::ActionEnum::Bet(amount.unwrap()?),
                "folds" => action::ActionEnum::Fold,
                "calls" => action::ActionEnum::Call,
                "raises" => action::ActionEnum::Raise(amount.unwrap()?),
                _ => {
                    return Err(PokerError::from_string(format!(
                        "Unknown action {}",
                        action_str
                    )))
                }
            };

            ret.push(PlayerAction {
                player_index,
                action,
                round,
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

    pub fn parse_cards<'a>(&'a self, s: &'a str) -> Result<(Vec<Card>, &str), PokerError> {
        let caps = self
            .cards_regex
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected cards in {}",
                &s[0..100]
            )))?;

        let cards_str = caps.get(0).unwrap().as_str();
        trace!("Cards: {} num matches {}", cards_str, caps.len());

        let just_cards_str = caps
            .get(1)
            .ok_or(PokerError::from_string(format!(
                "Expected cards in {}",
                &s[0..100]
            )))?
            .as_str();

        let cards: Vec<Card> = CardVec::try_from(just_cards_str)?.0;

        // let match_end = caps.get(0).unwrap().end();
        // let remaining_str = &s[match_end..];

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        Ok((cards, remaining_str))
    }

     pub fn parse_summary<'a>(
            &'a self,
            s: &mut &'a str,
            players: &Vec<InitialPlayerState>
        ) -> Result<&str, PokerError> {
            self.parse_section_name(s, "Summary")?;

            for _ in players.len() {
                let player_id = self.parse_player_id(s, Some(players));

                if player_id.is_err() {
                    //it's ok, not all players make it to the summary
                    break;
                }

                let player_id = player_id.unwrap();
                let wins_or_loses = self.parse_word(s)?;
                let amount = self.parse_chip_amount(s)?;
                trace!("Player {} won {}", player_id, amount);
            }

     }
}
