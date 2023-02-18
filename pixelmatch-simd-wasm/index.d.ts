/* tslint:disable */
/* eslint-disable */
/**
 * @param {Uint8Array} img1
 * @param {Uint8Array} img2
 * @param {number} width
 * @param {number} height
 * @param {Object} options
 * @returns {number}
 */
export function pixelmatch(
  img1: Uint8Array,
  img2: Uint8Array,
  width: number,
  height: number,
  options?: PixelmatchOptions
): { count: number; diff: Uint8Array };

export type PixelmatchOptions = {
  includeAntiAlias: boolean;
  threshold: number;
  diffColor: [number, number, number, number];
  antiAliasedColor: [number, number, number, number];
};
