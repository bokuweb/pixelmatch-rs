import path from "path";
import sharp from "sharp";

sharp("../examples/node/cjs/diff0.png").resize(800).toFile("../assets/diff0.png");
sharp("../examples/node/cjs/diff1.png").resize(800).toFile("../assets/diff1.png");
sharp("../examples/node/cjs/diff2.png").resize(800).toFile("../assets/diff2.png");
