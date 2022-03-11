interface Element {
  requestPaint(): void;
  paintContent(ctx: Graphics): void;
}

interface Graphics {
  lineCap: CanvasRenderingContext2D['lineCap'];
  lineJoin: CanvasRenderingContext2D['lineJoin'];
  strokeStyle: string | Graphics.Image;
  lineWidth: number;
  fillStyle: string | Graphics.Image;
  font: string;

  clearRect(x: number, y: number, w: number, h: number): void;
  beginPath(): void;
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
  bezierCurveTo(cp1x: number, cp1y: number, cp2x: number, cp2y: number, x: number, y: number): void;
  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, anticlockwise?: boolean): void;
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
  ellipse(
    x: number,
    y: number,
    radiusX: number,
    radiusY: number,
    rotation: number,
    startAngle: number,
    endAngle: number,
    anticlockwise?: boolean
  ): void;
  rect(x: number, y: number, width: number, height: number): void;
  closePath(): void;
  stroke(path?: Graphics.Path): void;
  fill(path?: Graphics.Path, fillRule?: 'nonzero' | 'evenodd'): void;
  strokeRect(x: number, y: number, width: number, height: number): void;
  fillRect(x: number, y: number, width: number, height: number): void;
  fillText(text: string, x: number, y: number, maxWidth: number): void;
  setLineDash(...segments: number[]): void;
  save(): void;
  restore(): void;
  scale(x: number, y: number): void;
  translate(x: number, y: number): void;
  rotate(radians: number): void;
  rotate(radians: number, x: number, y: number): void;
  transform(a: number, b: number, c: number, d: number, e: number, f: number): void;
  setTransform(a: number, b: number, c: number, d: number, e: number, f: number): void;

  draw(
    path: Graphics.Path,
    params: {
      x: number;
      y: number;
      fill?: 'nonzero' | 'evenodd';
      stroke?: boolean;
    }
  ): void;
  draw(
    image: Graphics.Image,
    params: {
      x: number;
      y: number;
      width?: number;
      height?: number;
      srcX?: number;
      srcY?: number;
      opacity?: number;
    }
  ): void;
  draw(
    text: Graphics.Text,
    params: {
      x: number;
      y: number;
      alignment: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;
      fill?: string;
    }
  ): void;
  // TODO: opacity and filter may not work in some situations.
  pushLayer(x: number, y: number, width: number, height: number): void;
  pushLayer(x: number, y: number, width: number, height: number, filter: string): void;
  pushLayer(
    clipAreaName: 'background-area' | 'border-box' | 'padding-box' | 'margin-box' | 'content-box',
    filter: string
  ): void;
  pushLayer(path: Graphics.Path): void;
  pushLayer(mask: Graphics.Image, useAlpha: boolean): void;
  popLayer(): void;
}

declare namespace Graphics {
  // TODO
  interface Image {}

  class Path {
    moveTo(x: number, y: number): void;
    lineTo(x: number, y: number): void;
    quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
    bezierCurveTo(cp1x: number, cp1y: number, cp2x: number, cp2y: number, x: number, y: number): void;
    arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, anticlockwise?: boolean): void;
    arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
    ellipse(
      x: number,
      y: number,
      radiusX: number,
      radiusY: number,
      rotation: number,
      startAngle: number,
      endAngle: number,
      anticlockwise?: boolean
    ): void;
    rect(x: number, y: number, width: number, height: number): void;
    closePath(): void;
    isPointInside(x: number, y: number): boolean;
    bounds(): [number, number, number, number];
    combine(how: 'union' | 'intersect' | 'xor' | 'exclude', otherPath: Path): Path;
  }

  class Text {
    constructor(s: string);

    readonly lines: number;
    chars: string;
    style: string;
    class: string;

    /**
     * Reports minimal, maximal and used width of the text block.
     */
    width(): [number, number, number];
    /**
     * Sets used width of the text block. Note: text.lines property may change after that.
     */
    width(usedWidth: number): void;
    /**
     * Reports content and used height of the text block.
     */
    height(): [number, number];
    /**
     * Sets used height of the text block. Note: vertical-align of text style may change location of glyphs on screen.
     */
    height(usedHeight: number): void;
    /**
     * @returns [yPos, height, baselineOffset]
     */
    lineMetrics(lineNo: number): [number, number, number] | undefined;
    lineChars(lineNo: number): string;
  }
}
