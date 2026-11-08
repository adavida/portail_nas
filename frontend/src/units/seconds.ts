import { scaledAmount } from "./scaled_amount";

const TIME_UNITS: Record<string, number> = { ms: 0.001, s: 1 };

export class Seconds {
  constructor(private readonly text: string) {}

  get value(): number {
    return scaledAmount(this.text, TIME_UNITS);
  }

  toString(): string {
    return this.text;
  }
}
