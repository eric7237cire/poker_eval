use super::{info_state_actions::NUM_ACTIONS, InfoStateActionValueType};

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