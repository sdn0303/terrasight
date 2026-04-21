import type { FeatureCollection } from "geojson";
import type { LandPriceAggregation } from "@/lib/api/schemas/land-price-aggregation";
import type { TransactionAggregation } from "@/lib/api/schemas/transaction-aggregation";

export const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

export const EMPTY_LAND_PRICE_AGG: LandPriceAggregation = {
  type: "FeatureCollection",
  features: [],
};

export const EMPTY_TRANSACTION_AGG: TransactionAggregation = {
  type: "FeatureCollection",
  features: [],
};
