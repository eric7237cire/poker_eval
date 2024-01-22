use serde::{Serialize, Deserialize};

use super::{info_state_actions::NUM_ACTIONS, InfoStateActionValueType};

#[derive(Serialize, Deserialize, Clone)]
pub struct InfoStateValue {
    /*
    sum == 1
    Current strategy profile for this info state
    */
    pub strategy: [InfoStateActionValueType; NUM_ACTIONS],

    //sum varies, negative values are 0'ed out
    //during an interation, we take utility of the infostate
    //so strategy * utility_of_each_action, so kind of a weighted average
    //regrets = utility of each action - utility of infostate
    //then this is added to this sum
    pub regret_sum: [InfoStateActionValueType; NUM_ACTIONS],

    /*
     sum == self.reach_pr_sum
     */
    pub strategy_sum: [InfoStateActionValueType; NUM_ACTIONS],
    
    // sum of probability that this node is reached, in all iterations
    pub reach_pr_sum: f64 
}

impl Default for InfoStateValue {
    fn default() -> Self {
        InfoStateValue {
            strategy: [1.0/NUM_ACTIONS as InfoStateActionValueType; NUM_ACTIONS],
            regret_sum: [0.0; NUM_ACTIONS],
            strategy_sum: [0.0; NUM_ACTIONS],
            reach_pr_sum: 0.0,
        }
    }
}