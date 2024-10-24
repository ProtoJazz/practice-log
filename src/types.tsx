import { Moment } from "moment";

export type Regiment = {
  id?: number;
  date: Moment;
  pieces: PracticePieces[];
};
export type PracticePieces = {
  id?: number;
  name: string;
  logs: PracticePieceLogs[];
};

export type PracticePieceLogs = {
  id?: number;
  practice_piece_id: number;
  bpm: number;
  timestamp: Moment;
};
