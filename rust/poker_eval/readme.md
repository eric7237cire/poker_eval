
Future dev ideas

MCCFR -- https://github.com/fedden/poker_ai

https://xyzml.medium.com/learn-ai-game-playing-algorithm-part-iii-counterfactual-regret-minimization-b182a7ec85fb

https://github.com/ArmanMielke/simple-poker-cfr?tab=readme-ov-file

https://rnikhil.com/2023/12/31/ai-cfr-solver-poker.html

https://dash.harvard.edu/bitstream/handle/1/37370951/thesis_submission.pdf?sequence=1

## Python implementation of Kuhn CFR

https://github.com/IanSullivan/PokerCFR

## Kuhn CFR Explation w/ math translation

https://justinsermeno.com/posts/cfr/

## C++ CFR Implemenatations

https://github.com/bakanaouji/cpp-cfr/blob/master/RegretMinimization/Trainer/Trainer.cpp

F# -- https://github.com/brianberns/Cfrm/tree/master/Cfrm

## Python Implementation

https://github.com/tansey/pycfr

Goal -- Human consumable advice




Vs flop 


Group together hole cards + flop into:

Totally missed
Dominating
Draws
Flop winning but vulnerable, top pair with flush draws in multiway
Potentially winning but nothing on the board (e.g. heads up with ace high)

Question:

Are dominating & draws represented in EQ ? 

First step is classify hole cards:

1. vs 1, 2, 3, 4, 5, 6, 7 players

Then evaluate vs all flops;
2. which are easily foldable, which have draw potential (river > flop), which are strong (flop ~~ river)

Do clustering on these to divide into groups/categories

3.  P(being dominated), e.g. AJ vs AT
Measure heads up 


Want to find best # of players + hole cards to have the 
best hand and the closest 2nd hand (most profitable)

Set a minimum for losing hand

Take a hole card,
deal randomly 3 other players

If winning hand below losing hand, drop

If hero hand < min, skip

Only 1 other hand above threshold (so no better hand on board)



######################################

Have classifier based on performance vs 4 players

# Node Definition:

3 -- Position (1st, last, middle)
4 -- Number of players (2, 3, 4+)
3 -- Hole cards (grouping into categories, because of # of players can make this into 3 groups maybe)
4 -- Round
3 -- Equity (high, medium, low)
3 -- Bet situation (unbet, facing bet, facing raise)

Actions:
Check, Bet/Raise, Fold, Call

Run through, just tracking gains/loses per node & action

# Tester

Be able to initialize the game runner up until a given point (via game log),
then try different actions,
and continue the game as normal

But....how to vary the opponents hole cards?  Knowing they might not have taken the same actions.

# Go through each hand where lost money,  won @ showdown, won with folding



# Hand Analyzer:

Detect -- not bet enough ?  
This is when an opponent had high equity (a good 2nd best hand) that we could have bet more
Detect: Find strong (better than top pair) 2nd best hands
Solution: Bet more

Detect -- Get outdrawn
This is high flop equity and loses on river
Detect by: Winners eq on flop or turn < hero equity 

Also too many players preflop/flop/turn/etc.

Solution: Bet higher preflop/flop/turn

Detect -- Got dominated 
Another player had better kicker, etc.
Solution: Preflop, bet less, fold


try_agents produces a csv that is analyzed with Analyze.ipynb

# Preflop precached data

Store in code for all hole card types (169)
The EQ vs random for 2-10 players


# CFR

Since our bots are deterministic, the probablity of reaching a node is 1 

So to train a decision/action at an information set, simulate the game with the same cards
for each action, adding the value in bb to each action.

The strategy is 

value of action / 
total value of all actions

