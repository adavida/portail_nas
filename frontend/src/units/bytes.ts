import { scaledAmount } from "./scaled_amount";

const SIZE_UNITS: Record<string, number> = {
  B: 1,
  KiB: 1024,
  MiB: 1024 ** 2,
  GiB: 1024 ** 3,
  TiB: 1024 ** 4,
};

export class Bytes {
  constructor(private readonly text: string) {}

  get value(): number {
    return scaledAmount(this.text, SIZE_UNITS);
  }

  toString(): string {
    return this.text;
  }
}
