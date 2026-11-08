import { scaledAmount } from "./scaled_amount";

const RATE_UNITS: Record<string, number> = {
  "B/s": 1,
  "KiB/s": 1024,
  "MiB/s": 1024 ** 2,
  "GiB/s": 1024 ** 3,
  "TiB/s": 1024 ** 4,
};

export class BytesPerSecond {
  constructor(private readonly text: string) {}

  get value(): number {
    return scaledAmount(this.text, RATE_UNITS);
  }

  toString(): string {
    return this.text;
  }
}
