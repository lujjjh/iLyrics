export type TimingFunction = (x: number) => number;

export const linear = (x: number) => x;
export const easeOutQuart = (x: number) => 1 - Math.pow(1 - x, 4);
