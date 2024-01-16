from pathlib import Path
from typing import Literal
import pandas as pd


def get_all_hands_df():
    csv_file_path = Path('/home/jovyan/work/hand_history.csv')

    
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file_path)

    df['STACK_DIFF'] = df['STACK_FINAL'] - df['STACK_PREFLOP']
    df['CALL_AMT_PREFLOP_PERC_POT'] = df['CALL_AMT_PREFLOP'] / df['POT_PREFLOP']
    df['CALL_AMT_FLOP_PERC_POT'] = df['CALL_AMT_FLOP'] / df['POT_FLOP']
    df['CALL_AMT_TURN_PERC_POT'] = df['CALL_AMT_TURN'] / df['POT_TURN']
    df['CALL_AMT_RIVER_PERC_POT'] = df['CALL_AMT_RIVER'] / df['POT_RIVER']

    df['WON'] = df['STACK_DIFF'] > 0

    return df 

# Get the original DataFrame indices for the first 10 rows of the filtered DataFrame
def print_first_10(filtered_df, desc: str):
    first_10_indices = filtered_df[["ITERATION_NUMBER", "STACK_DIFF"]].head(10)

    print(f"{desc} {len(filtered_df)}")
    print("Original indices of the first 10 rows:")
    print(first_10_indices)

def get_position(row, round_str: Literal['PREFLOP', 'FLOP', 'TURN', 'RIVER']):
    num_players = row[f'PLR_START_{round_str}']
    players_before = row[f'PLR_BEFORE_HERO_{round_str}']

    if players_before == 0:
        return 'First'
    elif players_before == num_players - 1:
        return 'Last'
    elif players_before <= num_players - 2:
        return 'In the Middle'
    else:
        return 'Invalid'    
    

# print(list(csv_file_path.parent.iterdir()))
# From the hole card analysis
STR_GROUPS =[ "AA, KK, QQ, JJ, TT" ,
    "AKs, AQs, AJs, ATs, A9s, A8s, AKo, KQs, KJs, KTs, AQo, KQo, AJo, ATo, A9o, 99, 88, 77, 66, 55",
    "A7s, A6s, A5s, A4s, A3s, A2s, K9s, K8s, K7s, K6s, K5s, K4s, K3s, QJs, QTs, Q9s, Q8s, Q7s, Q6s, KJo, QJo, JTs, J9s, J8s, KTo, QTo, JTo, T9s, K9o, Q9o, J9o, A8o, K8o, Q8o, A7o, K7o, A6o, K6o, A5o, K5o, A4o, 44, A3o, 33, A2o, 22" ,
    "K2s, Q5s, Q4s, Q3s, Q2s, J7s, J6s, J5s, J4s, J3s, J2s, T8s, T7s, T6s, T5s, T4s, T3s, T2s, T9o, 98s, 97s, 96s, 95s, 94s, J8o, T8o, 98o, 87s, 86s, 85s, Q7o, J7o, T7o, 97o, 87o, 76s, Q6o, J6o, T6o, 96o, Q5o, J5o, T5o, K4o, Q4o, J4o, K3o, Q3o, J3o, K2o, Q2o, J2o",
    "93s, 92s, 84s, 83s, 82s, 75s, 74s, 73s, 72s, 86o, 76o, 65s, 64s, 63s, 62s, 95o, 85o, 75o, 65o, 54s, 53s, 52s, T4o, 94o, 84o, 74o, 64o, 54o, 43s, 42s, T3o, 93o, 83o, 73o, 63o, 53o, 43o, 32s, T2o, 92o, 82o, 72o, 62o, 52o, 42o, 32o"]

GROUP_SETS = []
for group in STR_GROUPS:
    split_trimmed = [x.strip() for x in group.split(',')]
    GROUP_SETS.append(set(split_trimmed))
        
def get_group_set(simple_hole_cards: str):
    for i, group_set in enumerate(GROUP_SETS):
        if simple_hole_cards in group_set:
            return i
    return None 

def bin_perc(perc0_to_1: float, nearest=5) -> int:
    perc100 = perc0_to_1 * 100
    return int(round(perc100 / nearest) * nearest)


def bin_call_amount(call_amt, nearest=5) -> int:
    return int(round(call_amt / nearest) * nearest)
    
