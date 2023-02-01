let wasm;

(async () => {
  const response = await fetch("/benchmark/pixelmatch_simd_wasm_bg.wasm");
  const buf = await response.arrayBuffer();
  const wasmInstance = await (await WebAssembly.instantiate(buf)).instance;
  wasm = wasmInstance.exports;

  let cachedUint8Memory0 = new Uint8Array();
  function getUint8Memory0() {
    if (cachedUint8Memory0.byteLength === 0) {
      cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
  }

  function getImageDate(selector) {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    const img = document.querySelector(selector);
    canvas.width = img.width;
    canvas.height = img.height;
    context.drawImage(img, 0, 0);
    return context.getImageData(0, 0, img.width, img.height);
  }

  let WASM_VECTOR_LEN = 0;

  function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
  }

  const inner = function (
    img1,
    img2,
    out,
    width,
    height,
    include_anti_alias,
    threshold,
    diff_color_r,
    diff_color_g,
    diff_color_b,
    diff_color_a,
    anti_aliased_color_r,
    anti_aliased_color_g,
    anti_aliased_color_b,
    anti_aliased_color_a
  ) {
    try {
      const ptr0 = passArray8ToWasm0(img1, wasm.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passArray8ToWasm0(img2, wasm.__wbindgen_malloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = passArray8ToWasm0(out, wasm.__wbindgen_malloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = wasm.pixelmatch(
        ptr0,
        len0,
        ptr1,
        len1,
        ptr2,
        len2,
        width,
        height,
        include_anti_alias,
        threshold,
        diff_color_r,
        diff_color_g,
        diff_color_b,
        diff_color_a,
        anti_aliased_color_r,
        anti_aliased_color_g,
        anti_aliased_color_b,
        anti_aliased_color_a
      );
      return ret;
    } catch (e) {
      console.error(e);
    } finally {
      out.set(getUint8Memory0().subarray(ptr2 / 1, ptr2 / 1 + len2));
      wasm.__wbindgen_free(ptr2, len2 * 1);
    }
  };

  const createDefaultOptions = () => {
    return {
      includeAntiAlias: false,
      threshold: 0.1,
      diffColor: [255, 119, 119, 255],
      antiAliasedColor: [243, 156, 18, 255],
    };
  };

  const simd = (img1, img2, w, h, opts) => {
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
    return { count: countOrError, diff: out };
  };

  const suite = new Benchmark.Suite(`pixelmatch`);
  const out = new Uint8Array(800 * 578 * 4);
  const img1 = getImageDate(".before").data;
  const img2 = getImageDate(".after").data;
  suite
    .add("simd", {
      fn: () => {
        console.log(
          simd(img1, img2, 800, 574, {
            includeAntiAlias: false,
            threshold: 0.1,
          }).count
        );
      },
    })
    .add("js", {
      fn: () => {
        console.log(1 + 1);
      },
    })
    .on("complete", () => {
      console.log(
        "Fastest is " + suite.filter("fastest").map("name"),
        "js",
        suite[0].stats.mean
      );
    })
    .run();
})();