import { linear, TimingFunction } from './timing-function';

export class Transition {
  private startedAt: number;

  constructor(private duration: number, private timingFunction: TimingFunction = linear) {
    this.startedAt = Date.now();
  }

  get ended() {
    return Date.now() > this.startedAt + this.duration;
  }

  get currentRatio() {
    if (this.ended) return 1.0;
    return this.timingFunction((Date.now() - this.startedAt) / this.duration);
  }
}
