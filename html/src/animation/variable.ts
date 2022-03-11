import { TimingFunction } from './timing-function';
import { Transition } from './transition';

export class Variable {
  private previousValue: number;
  private nextValue: number;
  private currentTransition?: Transition;
  private raf?: number;
  private resolveTransitionPromise?: () => void;
  private lastValue?: number;
  private disposed = false;

  constructor(initialValue: number, private element?: Element) {
    this.previousValue = this.nextValue = initialValue;
  }

  set value(value) {
    if (this.raf) cancelAnimationFrame(this.raf);
    this.previousValue = this.nextValue = value;
  }

  get value() {
    const { previousValue, nextValue } = this;
    const ratio = this.currentTransition?.currentRatio ?? 1.0;
    let value = previousValue + ratio * (nextValue - previousValue);
    value = Math.floor(value * 1e3) / 1e3;
    return value;
  }

  private rafCallback = () => {
    const { lastValue, value, element, currentTransition } = this;
    if (lastValue === undefined || Math.abs(value - lastValue) > 1e-6) element?.requestPaint();
    this.lastValue = value;
    if (!currentTransition || currentTransition?.ended) {
      this.resolveTransitionPromise?.();
      this.raf = undefined;
    } else {
      this.raf = requestAnimationFrame(this.rafCallback);
    }
  };

  scheduleTransition(endValue: number, duration: number, timingFunction?: TimingFunction): Promise<void> {
    this.previousValue = this.value;
    this.nextValue = endValue;
    this.resolveTransitionPromise?.();
    this.currentTransition = new Transition(duration, timingFunction);
    let resolve: () => void;
    const promise = new Promise<void>((resolve_) => {
      resolve = resolve_;
    });
    this.resolveTransitionPromise = resolve!;
    if (!this.raf) this.raf = requestAnimationFrame(this.rafCallback);
    return promise;
  }

  dispose() {
    if (this.raf) cancelAnimationFrame(this.raf);
    this.resolveTransitionPromise?.();
  }
}
