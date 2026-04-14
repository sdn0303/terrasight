/**
 * Shared type definitions for the WASM SpatialEngine, used by both the
 * Web Worker and the adapter. Keep in sync with the generated
 * `public/wasm/realestate_wasm.d.ts`.
 */
export interface ISpatialEngine {
  load_layer(layer_id: string, fgb_bytes: Uint8Array): number;
  query(
    layer_id: string,
    south: number,
    west: number,
    north: number,
    east: number,
  ): string;
  query_layers(
    layer_ids: string,
    south: number,
    west: number,
    north: number,
    east: number,
  ): string;
  feature_count(layer_id: string): number;
  loaded_layers(): string;
  compute_stats(
    south: number,
    west: number,
    north: number,
    east: number,
  ): string;
  load_geojson_layer(layer_id: string, geojson: string): number;
  compute_tls(
    south: number,
    west: number,
    north: number,
    east: number,
    preset: string,
  ): string;
}

export interface IWasmModule {
  default: () => Promise<void>;
  SpatialEngine: new () => ISpatialEngine;
}

declare module "/wasm/realestate_wasm.js" {
  export default function init(): Promise<void>;
  export class SpatialEngine implements ISpatialEngine {
    constructor();
    load_layer(layer_id: string, fgb_bytes: Uint8Array): number;
    query(
      layer_id: string,
      south: number,
      west: number,
      north: number,
      east: number,
    ): string;
    query_layers(
      layer_ids: string,
      south: number,
      west: number,
      north: number,
      east: number,
    ): string;
    feature_count(layer_id: string): number;
    loaded_layers(): string;
    compute_stats(
      south: number,
      west: number,
      north: number,
      east: number,
    ): string;
    load_geojson_layer(layer_id: string, geojson: string): number;
    compute_tls(
      south: number,
      west: number,
      north: number,
      east: number,
      preset: string,
    ): string;
  }
}
