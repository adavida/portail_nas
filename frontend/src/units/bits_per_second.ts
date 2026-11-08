import { scaledAmount } from "./scaled_amount";

const BIT_UNITS: Record<string, number> = {
  "bit/s": 1,
  "Mib/s": 10 ** 6,
  "Gib/s": 10 ** 9,
};

export class BitsPerSecond {
  constructor(private readonly text: string) {}

  get value(): number {
    return scaledAmount(this.text, BIT_UNITS);
  }

  toString(): string {
    return this.text;
  }
}
