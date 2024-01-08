
Future dev ideas

MCCFR -- https://github.com/fedden/poker_ai

https://github.com/ArmanMielke/simple-poker-cfr?tab=readme-ov-file

https://rnikhil.com/2023/12/31/ai-cfr-solver-poker.html

https://dash.harvard.edu/bitstream/handle/1/37370951/thesis_submission.pdf?sequence=1


Goal -- Human consumable advice

Nodes:

Position (1st, last, middle)
Number of players (2, 3, 4, 5+)
Hole cards (grouping into categories)
Flop (unbet, facing bet, facing raise)
Turn (unbet, facing bet, facing raise)
River (unbet, facing bet, facing raise)
Equity (high, medium, low)


Decisions -- Preflop
3x, 10x, all in 
Decisions -- Postflop
Bet min, bet 1/3, 1/2, 1x pot , all in 
Raise 2x, to pot, all in

Vs flop 


Group together hole cards + flop into:

Totally missed
Dominating
Draws
Flop winning but vulnerable, top pair with flush draws in multiway
Potentially winning but nothing on the board (e.g. heads up with ace high)

Question:

Are dominating & draws represented in EQ ? 


Hand Analyzer:

Detect -- not bet enough ?  
This is when an opponent had high equity (a good 2nd best hand) that we could have bet more

Detect -- Get outdrawn
This is high flop equity and loses on river

Detect -- Got dominated 
Another player had better kicker, etc.


First step is classify hole cards:

1. vs 1, 2, 3, 4, 5, 6, 7 players

Then evaluate vs all flops;
2. which are easily foldable, which have draw potential (river > flop), which are strong (flop ~~ river)

Do clustering on these to divide into groups/categories

3.  P(being dominated), e.g. AJ vs AT
Measure heads up 