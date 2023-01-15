import { pixelmatch as inner } from "../../dist/web/pkg/pixelmatch_simd_wasm";

export const pixelmatch = (img1, img2, w, h, opts) => {
  const out = new Uint8Array(img1.length);
  const defaultOptions = createDefaultOptions();
  const diffColor = opts.diffColor ?? defaultOptions.diffColor;
  const antiAliasedColor =
    opts.antiAliasedColor ?? defaultOptions.antiAliasedColor;
  const countOrError = inner(
    img1,
    img2,
    out,
    w,
    h,
    opts.includeAntiAlias ?? defaultOptions.includeAntiAlias,
    opts.threshold ?? defaultOptions.threshold,
    diffColor[0],
    diffColor[1],
    diffColor[2],
    diffColor[3],
    antiAliasedColor[0],
    antiAliasedColor[1],
    antiAliasedColor[2],
    antiAliasedColor[3]
  );

  if (countOrError < 0) {
    if (countOrError === -1) {
      throw new ImageLengthError();
    } else if (countOrError === -2) {
      throw new InvalidFormatError();
    }
  }

  return { count: countOrError, diff: out };
};
