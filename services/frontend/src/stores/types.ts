export interface BBox {
	south: number;
	west: number;
	north: number;
	east: number;
}

export type WeightPreset =
	| "balance"
	| "investment"
	| "residential"
	| "disaster";
