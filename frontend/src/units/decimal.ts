export class Decimal {
  constructor(private readonly text: string) {}

  get value(): number {
    return Number.parseFloat(this.text);
  }

  toString(): string {
    return this.text;
  }
}
