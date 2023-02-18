export const createDefaultOptions = () => {
  return {
    includeAntiAlias: false,
    threshold: 0.1,
    diffColor: [255, 119, 119, 255],
    antiAliasedColor: [243, 156, 18, 255],
  };
};

export class ImageLengthError extends Error {
  constructor() {
    super("input buf length error. please input same length images");
    this.name = "ImageLengthError";
  }
}

export class InvalidFormatError extends Error {
  constructor() {
    super("input buf format error. please input RGBA 24bit image data");
    this.name = "InvalidFormatError";
  }
}
