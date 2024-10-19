import { Moment } from "moment";

export type Regiment = {
  id: string;
  date: Moment;
  pieces: PracticePieces[];
};
export type PracticePieces = {
  id: string;
  name: string;
};
