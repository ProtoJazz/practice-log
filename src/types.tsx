import { Moment } from "moment";

export type Regiment = {
  id?: number;
  date: Moment;
  pieces: PracticePieces[];
};
export type PracticePieces = {
  id?: number;
  name: string;
};
