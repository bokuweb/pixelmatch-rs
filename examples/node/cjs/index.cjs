const { pixelmatch } = require("@bokuweb/pixelmatch-wasm");

const { readFileSync, createWriteStream } = require("fs");
const { join } = require("path");
const { PNG } = require("pngjs");

for (let i = 0; i < 3; i++) {
  const path1 = join(__dirname, `../../../fixtures/00${i}a.png`);
  const img1 = PNG.sync.read(readFileSync(path1));
  const path2 = join(__dirname, `../../../fixtures/00${i}b.png`);
  const img2 = PNG.sync.read(readFileSync(path2));

  const res = pixelmatch(img1.data, img2.data, img1.width, img1.height, {
    includeAntiAlias: true,
    threshold: 0.1,
  });

  let diff = new PNG({ width: img1.width, height: img1.height });
  diff.data = res.diff;
  diff.pack().pipe(createWriteStream(`diff${i}.png`));
}
