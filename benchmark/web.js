let simd;
let without;

(async () => {
  const response = await fetch(
    "/pixelmatch-simd-wasm/dist/web/pkg/pixelmatch_simd_wasm_bg.wasm"
  );
  const buf = await response.arrayBuffer();
  const wasmInstance = await (await WebAssembly.instantiate(buf)).instance;
  simd = wasmInstance.exports;

  const response1 = await fetch(
    "/pixelmatch-wasm/dist/web/pkg/pixelmatch_wasm_bg.wasm"
  );
  const buf1 = await response1.arrayBuffer();
  const wasmInstance1 = await (await WebAssembly.instantiate(buf1)).instance;
  without = wasmInstance1.exports;

  function getImageDate(selector) {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    const img = document.querySelector(selector);
    canvas.width = img.width;
    canvas.height = img.height;
    context.drawImage(img, 0, 0);
    return [
      context.getImageData(0, 0, img.width, img.height),
      img.width,
      img.height,
    ];
  }

  const simdInner = function (
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
    let WASM_VECTOR_LEN = 0;
    let cachedUint8Memory0 = new Uint8Array();

    function getUint8Memory0() {
      if (cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(simd.memory.buffer);
      }
      return cachedUint8Memory0;
    }

    function passArray8ToWasm0(arg, malloc) {
      const ptr = malloc(arg.length * 1);
      getUint8Memory0().set(arg, ptr / 1);
      WASM_VECTOR_LEN = arg.length;
      return ptr;
    }

    try {
      const ptr0 = passArray8ToWasm0(img1, simd.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passArray8ToWasm0(img2, simd.__wbindgen_malloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = passArray8ToWasm0(out, simd.__wbindgen_malloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = simd.pixelmatch(
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
      simd.__wbindgen_free(ptr2, len2 * 1);
    }
  };

  const withoutInner = function (
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
    let WASM_VECTOR_LEN = 0;
    let cachedUint8Memory0 = new Uint8Array();

    function getUint8Memory0() {
      if (cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(without.memory.buffer);
      }
      return cachedUint8Memory0;
    }

    function passArray8ToWasm0(arg, malloc) {
      const ptr = malloc(arg.length * 1);
      getUint8Memory0().set(arg, ptr / 1);
      WASM_VECTOR_LEN = arg.length;
      return ptr;
    }

    try {
      const ptr0 = passArray8ToWasm0(img1, without.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passArray8ToWasm0(img2, without.__wbindgen_malloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = passArray8ToWasm0(out, without.__wbindgen_malloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = without.pixelmatch(
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
      without.__wbindgen_free(ptr2, len2 * 1);
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

  const simdMatch = (img1, img2, w, h, opts) => {
    const out = new Uint8Array(img1.length);
    const defaultOptions = createDefaultOptions();
    const diffColor = opts.diffColor ?? defaultOptions.diffColor;
    const antiAliasedColor =
      opts.antiAliasedColor ?? defaultOptions.antiAliasedColor;
    const countOrError = simdInner(
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
      antiAliasedColor[3],
      simd
    );
    return { count: countOrError, diff: out };
  };

  const withoutMatch = (img1, img2, w, h, opts) => {
    const out = new Uint8Array(img1.length);
    const defaultOptions = createDefaultOptions();
    const diffColor = opts.diffColor ?? defaultOptions.diffColor;
    const antiAliasedColor =
      opts.antiAliasedColor ?? defaultOptions.antiAliasedColor;
    const countOrError = withoutInner(
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
      antiAliasedColor[3],
      without
    );
    return { count: countOrError, diff: out };
  };

  for (let i = 0; i < 3; i++) {
    const suite = new Benchmark.Suite(`pixelmatch`);
    const [{ data: img1 }, w, h] = getImageDate(`.before${i}`);
    const [{ data: img2 }] = getImageDate(`.after${i}`);
    suite
      .add("simd", {
        fn: () => {
          simdMatch(img1, img2, w, h, {
            includeAntiAlias: false,
            threshold: 0.1,
          });
        },
      })
      .add("without", {
        fn: () => {
          withoutMatch(img1, img2, w, h, {
            includeAntiAlias: false,
            threshold: 0.1,
          });
        },
      })
      .add("js", {
        fn: () => {
          const out = new Uint8Array(img1.length);
          pixelmatch(img1, img2, out, w, h, {
            includeAA: false,
            threshold: 0.1,
          });
        },
      })
      .on("complete", () => {
        console.log(
          "Fastest is " + suite.filter("fastest").map("name"),
          ", simd",
          suite[0].stats.mean,
          "without",
          suite[1].stats.mean,
          "js",
          suite[2].stats.mean
        );
        const result = document.querySelector(`div.result${i}`);
        result.textContent = `Fastest is ${suite.filter("fastest").map("name")}
        simd = ${suite[0].stats.mean}
        without = ${suite[1].stats.mean}
        js = ${suite[2].stats.mean}
        `;
      })
      .run();
  }
})();
