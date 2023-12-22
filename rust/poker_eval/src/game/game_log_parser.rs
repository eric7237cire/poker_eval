use crate::{HoleCards, CardVec};
use log::trace;
use regex::Regex;

use crate::{Card, ChipType, InitialPlayerState, PlayerAction, PokerError, Round};

use super::action;

pub struct GameLogParser {
    pub section_name_regex: Regex,
    
    pub cards_regex: Regex,
    
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
(?:\#[^\n\r]*)?         # Non-capturing group for # and anything after it, greedy
\s*
\b{start-half}          # Non consuming word boundary
(?P<player_id>[\w\ ]+)  # Capture player id
\b{end-half}            # Non consuming word boundary
\ +
((wins|loses|bets|raises|calls|folds|checks|-))  # Capture ending
"#).unwrap(),
chip_amount_regex: Regex::new(r#"(?x) # Enable verbose mode
^\s*                    # Asserts the start of the string and matches any leading whitespace
(?P<amount>[\d\.\,]+)   # Capture amount
\b{end-half}            # Non consuming word boundary
"#).unwrap(),
                      

            section_name_regex: Regex::new(
                r#"(?x) # Enable verbose mode            
    ^\s*                    # Asserts the start of the string and matches any leading whitespace
    (?:\#[^\n\r]*)?         # Non-capturing group for # and anything after it, greedy
    \s*
    \*+\s*                  # Matches one or more '*' followed by any whitespace
    (?P<section_name>.+?)   # Lazily captures one or more characters as the section name
    \s*\*+                  # Matches any trailing whitespace followed by one or more '*'
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
                ([\w]+)                 # A word
                \b{end-half}            # Non consuming word boundary
                "#).unwrap(),

        }
    }

    //returns player_id and remaining string
    //if players is passed, will make sure player exists
    pub fn parse_player_id<'a>(&'a self, remaining_str: &mut &'a str, players: Option<&Vec<InitialPlayerState>>,) -> Result<&str, PokerError> {
        let caps = self
            .player_id_regex
            .captures(remaining_str)
            .ok_or(PokerError::from_string(format!(
                "Expected player id in [{:.100}]", &remaining_str
            )))?;

        let player_id = caps.name("player_id").unwrap().as_str();
        trace!("Player id: {}", player_id);

        //the remaining str is beginning of group 2
        let match_start = caps.get(2)
        .ok_or(PokerError::from_string(format!(
            "Expected player id *** player id ***"
        )))?
        .start();
        *remaining_str = &remaining_str[match_start..];

        if let Some(players) = players {
            if !players.iter().any(|p| p.player_name == player_id) {
                return Err(PokerError::from_string(format!(
                    "Expected player id [{}] to be in players",
                    player_id
                )));
            }
        }

        

        Ok(player_id)
    }

    pub fn parse_word<'a>(&'a self, s: &mut &'a str) -> Result<&str, PokerError> {
        let caps = self
            .get_word
            .captures(s)
            .ok_or(PokerError::from_string(format!(
                "Expected a word in {}"
            , &s[0..100])))?;

        let match_end = caps.get(0).unwrap().end();
        let remaining_str = &s[match_end..];

        trace!(
            "Remaining string len: {} start {}",
            remaining_str.len(),
            &remaining_str[0..10]
        );

        *s = remaining_str;

        //If the regex passed we should definitely have something in group1
        let word = caps.get(1).ok_or(PokerError::from_string(format!(
            "Expected word"
        )))?.as_str();
        Ok(word)
    }

    

    pub fn parse_dash<'a>(&'a self, s: &mut &'a str, expected: bool) -> Result<bool,PokerError> {
        
        for c in s.chars() {
            if c == '-' {
                *s = &s[1..];
                return Ok(true);
            } else if c == ' ' {
                *s = &s[1..];
                continue;
            } else {
                break;
            }
        }
        if expected {
            return Err(PokerError::from_string(format!(
                "Expected dash in {}", &s[0..100]
            )));
        }
        Ok(false)
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
            "Remaining string len: {} start {:.100}",
            remaining_str.len(),
            &remaining_str
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
                "Expected section *** section name *** in {:.100}", &s
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
        remaining_str: &mut &'a str,
    ) -> Result<Vec<InitialPlayerState>, PokerError> {

        let _section_name = self.parse_section_name( remaining_str, Some("Players"))?;

        
        let mut ret = Vec::new();
        
        // Cards are optional
        // Plyr B - 147 - 2d 2c
        // Plyr C - 98
        for _ in 0..15 {
            let player_id = self.parse_player_id(remaining_str, None);

            if player_id.is_err() {
                //maybe its the next section
                break;
            }
            let player_id = player_id.unwrap();

            self.parse_dash(remaining_str, true)?;
            let stack = self.parse_chip_amount(remaining_str)?;

            let dash = self.parse_dash(remaining_str, false)?;

            let mut cards: Option<HoleCards> = None;

            if dash {
                //Parse optional cards only if second dash is there
                let vec_cards = self.parse_cards(remaining_str)?;
                
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

        Ok(ret)
    }

    pub fn parse_blinds<'a>(
        &'a self,
        players: &Vec<InitialPlayerState>,
        remaining_str: &mut &'a str,
    ) -> Result<(ChipType, ChipType), PokerError> {

        self.parse_section_name(remaining_str, Some("Blinds"))?;
        
        let p1_name = self.parse_player_id(remaining_str, Some(players))?;
        self.parse_dash(remaining_str, true)?;
        let sb = self.parse_chip_amount(remaining_str)?;
        let p2_name = self.parse_player_id(remaining_str, Some(players))?;
        self.parse_dash(remaining_str, true)?;
        let bb = self.parse_chip_amount(remaining_str)?;

        

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

        

        Ok((sb, bb))
    }

    pub fn get_player_index(players: &Vec<InitialPlayerState>, player_id: &str) -> Result<usize, PokerError> {
        players
            .iter()
            .position(|p| p.player_name == player_id)
            .ok_or(PokerError::from_string(format!(
                "Could not find player [{}]",
                player_id
            )))
    }

    pub fn parse_round_actions<'a>(
        &'a self,
        players: &Vec<InitialPlayerState>,
        round: Round,
        remaining_str: &mut &'a str,
    ) -> Result<Vec<PlayerAction>, PokerError> {
        trace!("Parsing round actions for round {}", round.to_string());
        let mut ret = Vec::new();
        
        for _ in 0..ACTION_LIMIT {
           
            let player_id = self.parse_player_id(remaining_str, Some(players));
            
            if !player_id.is_ok() {
                break;
            }
            let player_id = player_id.unwrap();
            trace!("Player id: {}", player_id);

            //lookup index of player or return error if we don't find it
            let player_index = Self::get_player_index(players, player_id)?;

            let action_str = self.parse_word(remaining_str)?;
            //trace!("Action: {}", action_str);

            let action = match action_str {
                "checks" => action::ActionEnum::Check,
                "bets" => action::ActionEnum::Bet(self.parse_chip_amount(remaining_str)?),
                
                "folds" => action::ActionEnum::Fold,
                "calls" => action::ActionEnum::Call,
                "raises" => action::ActionEnum::Raise(self.parse_chip_amount(remaining_str)?),
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

            trace!("Parsed {}", ret.last().unwrap());
        }

        Ok(ret)
    }

    pub fn parse_cards<'a>(&'a self, s: &mut &'a str) -> Result<Vec<Card>, PokerError> {
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

        *s = remaining_str;

        Ok(cards)
    }

     pub fn parse_summary<'a>(
            &'a self,
            remaining_str: &mut &'a str,
            players: &Vec<InitialPlayerState>
        ) -> Result<Vec<ChipType>, PokerError> {
            trace!("Parsing summary");
            let mut ret = vec![0 as ChipType; players.len()];

            //We may already parsed this
            self.parse_section_name(remaining_str, Some("Summary"));

            
            for _ in 0..players.len() {
                let player_id = self.parse_player_id(remaining_str, Some(players));

                if player_id.is_err() {
                    //it's ok, not all players make it to the summary
                    break;
                }

                let player_id = player_id.unwrap();
                self.parse_dash(remaining_str, true)?;
                let amount = self.parse_chip_amount(remaining_str)?;

                let player_index = Self::get_player_index(players, player_id)?;

                ret[player_index] = amount;
            }

            Ok(ret)

     }
}
