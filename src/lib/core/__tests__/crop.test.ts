import { describe, expect, it } from "vitest";
import { cropRegion, regionCenter } from "../engine";

describe("cropRegion", () => {
  it("calculates a single-step crop for a 3x3 grid", () => {
    const region = { x: 0, y: 0, width: 300, height: 300 };
    const next = cropRegion(region, 3, 3, 5);

    expect(next).toEqual({ x: 100, y: 100, width: 100, height: 100 });
  });

  it("returns the center point with rounding", () => {
    const region = { x: 10, y: 10, width: 5, height: 5 };
    const center = regionCenter(region);

    expect(center).toEqual({ x: 13, y: 13 });
  });
});
