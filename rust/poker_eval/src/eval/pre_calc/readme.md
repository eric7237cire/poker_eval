# Ideas

Drawing upon https://github.com/b-inary/holdem-hand-evaluator

We either eval the hand with the unique sum of the card values,
and if it's suited then we just need the value of those suited cards (13 bits)


For 5 to 7 cards, there are only 73,775 unique sums of the card values

We test weights from         
https://github.com/zekyll/OMPEval/blob/master/omp/HandEvaluator.cpp 
and 
https://github.com/b-inary/holdem-hand-evaluator/blob/main/assets/src/constants.rs

Which use 25 bits max to store this sum

The weights work like
num of 2s (index 0) * weights[0]
+ num of 3s (index 1) * weights[1]
+ num of 4s (index 2) * weights[2]
...
+ num of Aces (index 12) * weights[12]

This sum is unique

The 'naive' solution, is to use base 5 for the weights
so
weights[0] = 1
weights[1] = 5
weights[2] = 25
weights[3] = 5^3 = 125
...
weights[12] = 5^12 = 244_140_625

this is a bit wasteful, but I couldn't figure out the secret magic with the better bases,
and this one will use max 5^13 - 1 = 1,220,703,124 which fits in 31 bits

Next step is to convert these 31 bit values into an index, which is a great use of a perfect hash function


The library ph can create a perfect hashing function, and it turns out, there
is not much benefit to condencing the unhashed weights 

These card value sums are all you need to determine hand rank for non flush hands

For flushes, we need 3 bits per suit, to store a max count of 8 
let flush_key = (self.mask >> (4 * is_flush.leading_zeros())) as u16;

we need at least 5, so this is that 3rd bit

000 000 000 000 

An improvement though is we want the mask to tell us if we have a flush, so we initialize the
count to 3, and check for 8, so we need 4 bits for this

0011 0011 0011 0011 (0x3333)

showing 0 1 2 3 5 6 7 of the suit
0 - 0011
1 - 0100
2 - 0101
3 - 0110
4 - 0111
5 - 1000
6 - 1001
7 - 1010
8 - 1011 (the max)

so this is how we can just check the mask
1000 1000 1000 1000 (0x8888)

# Eval

rank_weights = [1, 5, 25, 125, 5^4, ... 5^12]
suit_weights = [1 << 12, 1 << 8, 1 << 4, 1 << 0]  (all these shifted 31 bits; SUIT_SHIFT)

we need 31 bits for the card values, and 16 bits for the flush counts

so to evaluate cards, we need to --

lookup_key 
For each card, add rank_weights[card_value] to the lookup_key

We also add it's suit to the lookup key

We also add its bit mask, which groups together suits, etc
cccccccc sssssssss hhhhhhh dddddd 


 let flush_key = (self.mask >> (4 * is_flush.leading_zeros())) as u16;

 So recall the flush mask will give us a single bit, in suit_shift + 3, 7, 11, 15

 This flush key is basically 7 bits
