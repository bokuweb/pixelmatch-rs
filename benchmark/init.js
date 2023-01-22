function getImageDate(selector) {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    const img = document.querySelector(selector);
    canvas.width = img.width;
    canvas.height = img.height;
    context.drawImage(img, 0, 0);
    return context.getImageData(0, 0, img.width, img.height);
  }
  
  async function fetchExposedFunction(wasmFile) {
    let memory = new WebAssembly.Memory({ initial: 256 });
    const response = await fetch(wasmFile);
    const buf = await response.arrayBuffer();
    const obj = await WebAssembly.instantiate(buf, { env: { memory } });
    const exports = obj.instance.exports;
    if (exports.memory) {
      memory = exports.memory;
    }
    const before = getImageDate(".before").data;
    const after = getImageDate(".after").data;
    const bytesPerImage = before.length;
    const minimumMemorySize = bytesPerImage * 3;
    const pagesNeeded = Math.ceil(minimumMemorySize / (64 * 1024));
    memory.grow(pagesNeeded * 4);
    const mem = new Uint8ClampedArray(memory.buffer);
    mem.set(before);
    mem.set(after, before.length);
    return exports.a || exports.pixelmatch;
  }
  
  (async () => {
    const cpp_opt = await fetchExposedFunction("./cpp/cpp_opt.wasm");
    const cpp = await fetchExposedFunction("./cpp/pixelmatch.wasm");
    const rust = await fetchExposedFunction(
      "./rust/pkg/pixelmatch_optimized.wasm"
    );
    const as = await fetchExposedFunction(
      "./assemblyscript/build/optimized_opt.wasm"
    );
    const asInline = await fetchExposedFunction(
      "./assemblyscript/build/optimized_inline_opt.wasm"
    );
    const suite = new Benchmark.Suite(`cpp`);
    const out = new Uint8Array(800 * 578 * 4);
    const offset1 = 800 * 578 * 4;
    const offset2 = offset1 * 2;
    const before = getImageDate(".before").data;
    const after = getImageDate(".after").data;
    suite
      .add("cpp", {
        fn: () => cpp(0, offset1, 800, 578, offset2)
      })
      .add("cpp_opt", {
        fn: () => cpp_opt(0, offset1, 800, 578, offset2)
      })    
      .add("rust", {
        fn: () => rust(0, offset1, 800, 578, offset2)
      })
      .add("as", {
        fn: () => as(0, offset1, 800, 578, offset2)
      })
      .add("asInline", {
        fn: () => asInline(0, offset1, 800, 578, offset2)
      })
      .add("js", {
        fn: () => pixelmatch(before, after, out, 800, 578)
      })
      .on("cycle", event => {
        console.log(String(event.target));
      })
      .on("complete", () => {
        console.log(
          "Fastest is " + suite.filter("fastest").map("name"),
          suite[0].stats.mean,
          suite[1].stats.mean
        );
      })
      .run();
  })();