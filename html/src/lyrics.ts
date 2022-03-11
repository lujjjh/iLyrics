import { easeOutQuart, Variable } from './animation';

class Frame {
  private static sizeTransitionDuration = 200;
  private static sizeTransitionTimingFunction = easeOutQuart;
  private static opacityTransitionDuration = 500;
  private static opacityTransitionTimingFunction = easeOutQuart;

  private width: Variable;
  private height: Variable;
  private opacity: Variable;

  constructor(private element: Element) {
    this.width = new Variable(0, element);
    this.height = new Variable(0, element);
    this.opacity = new Variable(0, element);
  }

  setSize(width: number, height: number) {
    this.width.scheduleTransition(width, Frame.sizeTransitionDuration, Frame.sizeTransitionTimingFunction);
    this.height.scheduleTransition(height, Frame.sizeTransitionDuration, Frame.sizeTransitionTimingFunction);
  }

  setOpacity(value: number) {
    this.opacity.scheduleTransition(value, Frame.opacityTransitionDuration, Frame.opacityTransitionTimingFunction);
  }

  render(ctx: Graphics, renderContent: (ctx: Graphics) => void) {
    const clientRect = this.element.getBoundingClientRect();
    const width = this.width.value;
    const height = this.height.value;
    const left = clientRect.width / 2 - width / 2;
    const top = clientRect.height / 2 - height / 2;
    const right = left + width;
    const bottom = top + height;
    const radius = 4;
    const path = new Graphics.Path();
    path.moveTo(right, bottom);
    path.arcTo(left, bottom, left, top, radius);
    path.arcTo(left, top, right, top, radius);
    path.arcTo(right, top, right, bottom, radius);
    path.arcTo(right, bottom, left, bottom, radius);
    const opacity = this.opacity.value;
    // HACK: opacity(1) does not work
    ctx.pushLayer('content-box', opacity < 1 ? `opacity(${opacity})` : '');
    ctx.pushLayer(path);
    ctx.save();
    ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
    ctx.fillRect(left, top, right, bottom);
    ctx.restore();
    renderContent(ctx);
    ctx.popLayer();
    ctx.popLayer();
  }
}

class Line {
  private static opacityTransitionDuration = 500;
  private static opacityTransitionTimingFunction = easeOutQuart;
  private static translateTransitionDuration = 500;
  private static translateTransitionTimingFunction = easeOutQuart;

  private text: Graphics.Text;
  private opacity: Variable;
  private translateY: Variable;

  private visible = false;

  constructor(private element: Element, text: string) {
    this.text = new Graphics.Text(text, 'lyrics-line');
    this.opacity = new Variable(0, element);
    this.translateY = new Variable(0.5 * this.height, element);
  }

  get width() {
    return this.text.width()[2];
  }

  get height() {
    return this.text.height()[0];
  }

  render(ctx: Graphics) {
    const { width, height } = this.element.getBoundingClientRect();
    const opacity = this.opacity.value;
    const translateY = this.translateY.value;
    ctx.pushLayer('content-box', opacity < 1 ? `opacity(${opacity})` : '');
    ctx.draw(this.text, { x: width / 2, y: height / 2 + translateY, alignment: 5 });
    ctx.popLayer();
  }

  async show() {
    if (this.visible) return;
    this.visible = true;
    return Promise.all([
      this.opacity.scheduleTransition(1, Line.opacityTransitionDuration, Line.opacityTransitionTimingFunction),
      this.translateY.scheduleTransition(1, Line.translateTransitionDuration, Line.translateTransitionTimingFunction),
    ]);
  }

  async hide() {
    if (!this.visible) return;
    this.visible = false;
    return Promise.all([
      this.opacity.scheduleTransition(0, Line.opacityTransitionDuration, Line.opacityTransitionTimingFunction),
      this.translateY.scheduleTransition(
        -0.5 * this.height,
        Line.translateTransitionDuration,
        Line.translateTransitionTimingFunction
      ),
    ]);
  }

  dispose() {
    this.opacity.dispose();
    this.translateY.dispose();
  }
}

export class Lyrics extends Element {
  private static maxActiveLines = 3;
  private static frameHorizontalPadding = 10;

  private frame = new Frame(this);
  private activeLines: Line[] = [];

  setLyricsLine(s: string) {
    if (this.activeLines.length >= Lyrics.maxActiveLines) {
      const deleted = this.activeLines.splice(0, this.activeLines.length - Lyrics.maxActiveLines + 1);
      deleted.forEach((line) => line.dispose());
    }
    for (const line of this.activeLines) {
      line.hide().then(() => {
        // TODO: performance?
        this.activeLines = this.activeLines.filter((x) => x !== line);
      });
    }
    if (s === '') {
      this.frame.setOpacity(0);
    } else {
      const line = new Line(this, s);
      this.frame.setSize(line.width + Lyrics.frameHorizontalPadding * 2, line.height);
      this.frame.setOpacity(1);
      line.show();
      this.activeLines.push(line);
    }
  }

  paintContent(ctx: Graphics) {
    this.frame.render(ctx, () => {
      for (const line of this.activeLines) {
        line.render(ctx);
      }
    });
  }
}

export const initWindow = () => {
  const updateWindow = () => {
    const window = Window.this;
    window.isTopmost = true;
    const height = 100;
    const [left, _top, right, bottom] = window.screenBox('workarea', 'rect', false);
    window.move(left, bottom - height, right - left, height);
  };

  Window.this.addEventListener('spacechange', () => void updateWindow());

  updateWindow();

  const trayIcon = Window.this.trayIcon({
    text: 'iLyrics',
  });

  Window.this.on('trayiconclick', (evt) => {
    var [sx, sy] = Window.this.box('position', 'client', 'screen', true);
    var menu = document.$('menu#tray');
    var { screenX, screenY } = evt.data;
    menu.popupAt(screenX - sx, screenY - sy, 2);
  });

  globalThis.setLyrics = (text: string) => {
    document.querySelector('lyrics').setLyricsLine(text);
  };
};
