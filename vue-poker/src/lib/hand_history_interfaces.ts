export interface HandHistory {
  players: Player[];
  sb: number;
  bb: number;
  board: Board[];
  actions: Action[];
  final_stacks: number[];
}

export interface Player {
  stack: number;
  player_name: string;
  position: Position;
  cards: Cards;
}

export interface Position {
  pos: number;
}

export interface Cards {
  card_hi_lo: CardHiLo[];
}

export interface CardHiLo {
  value: string;
  suit: string;
  index: number;
}

export interface Board {
  value: string;
  suit: string;
  index: number;
}

export interface Action {
  player_index: number;
  action: FoldAction | RaiseAction | BetAction | CallAction | CheckAction;
  round: string;
  player_comment: string;
  pot: number;
  current_amt_to_call: number;
  amount_put_in_pot_this_round: number;
  total_amount_put_in_pot: number;
  players_left_to_act: number;
  is_all_in: boolean;
}

export type FoldAction = 'Fold';
export type CheckAction = 'Check';

export interface RaiseAction {
  Raise: [number, number];
}
export interface BetAction {
  Bet: number;
}
export interface CallAction {
  Call: number;
}
