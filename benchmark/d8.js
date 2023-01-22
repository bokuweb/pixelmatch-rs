const buf = read(
  "../pixelmatch-simd-wasm/dist/node/pkg/pixelmatch_simd_wasm_bg.wasm",
  "binary"
);

WebAssembly.instantiate(buf).then(({ instance }) => {
  const { pixelmatch } = instance.exports;
  const res = pixelmatch(
    new Uint8Array([255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    new Uint8Array([0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    2,
    2,
    {
      includeAntiAlias: true,
      threshold: 0.2,
    }
  );
  console.log(res);
});
