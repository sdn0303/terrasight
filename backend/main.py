import hashlib
import json
import logging
import os
from contextlib import asynccontextmanager

import httpx
from dotenv import load_dotenv
from fastapi import FastAPI, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse

from lib.cache import get_cached, init_cache, set_cached
from lib.tile_math import bbox_to_tiles, get_surrounding_tiles

load_dotenv()

API_KEY = os.getenv("REINFOLIB_API_KEY", "")
CACHE_TTL = int(os.getenv("CACHE_TTL_SECONDS", "86400"))
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO")

logging.basicConfig(level=getattr(logging, LOG_LEVEL, logging.INFO))
logger = logging.getLogger(__name__)

BASE_URL = "https://www.reinfolib.mlit.go.jp/ex-api/external"


@asynccontextmanager
async def lifespan(app: FastAPI):
    await init_cache()
    logger.info("Cache initialized")
    yield


app = FastAPI(title="MLIT Real Estate API Proxy", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_methods=["*"],
    allow_headers=["*"],
)
app.add_middleware(GZipMiddleware, minimum_size=500)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

async def fetch_reinfolib(endpoint: str, params: dict) -> dict:
    """Fetch from MLIT API with caching."""
    cache_key = hashlib.sha256(
        f"{endpoint}:{json.dumps(params, sort_keys=True)}".encode()
    ).hexdigest()

    cached = await get_cached(cache_key, CACHE_TTL)
    if cached is not None:
        logger.debug("Cache hit: %s", cache_key[:12])
        return cached

    url = f"{BASE_URL}/{endpoint}"
    headers = {"Ocp-Apim-Subscription-Key": API_KEY}

    async with httpx.AsyncClient(timeout=30.0) as client:
        resp = await client.get(url, params=params, headers=headers)
        resp.raise_for_status()
        data = resp.json()

    await set_cached(cache_key, data)
    return data


async def fetch_tile_geojson(
    endpoint: str,
    tiles: list[tuple[int, int]],
    extra_params: dict | None = None,
    zoom: int = 15,
) -> dict:
    """Fetch multiple tiles, merge features, deduplicate by coordinate string."""
    all_features: list[dict] = []
    seen_coords: set[str] = set()

    for x, y in tiles:
        params = {"response_format": "geojson", "z": zoom, "x": x, "y": y}
        if extra_params:
            params.update(extra_params)
        try:
            data = await fetch_reinfolib(endpoint, params)
            for feature in data.get("features", []):
                coord_key = json.dumps(feature.get("geometry", {}).get("coordinates"))
                if coord_key not in seen_coords:
                    seen_coords.add(coord_key)
                    all_features.append(feature)
        except Exception as exc:
            logger.warning("Failed to fetch tile %s/%s/%s: %s", zoom, x, y, exc)

    return {"type": "FeatureCollection", "features": all_features}


# ---------------------------------------------------------------------------
# Endpoints
# ---------------------------------------------------------------------------

@app.get("/api/health")
async def health():
    return {"status": "ok", "api_key_set": bool(API_KEY)}


@app.get("/api/transactions")
async def transactions(
    city: str = Query(...),
    year: str = Query(...),
    price_type: str = Query("02"),
):
    data = await fetch_reinfolib("XIT001", {
        "city": city,
        "year": year,
        "priceClassification": price_type,
    })
    return JSONResponse(content=data)


@app.get("/api/landprice")
async def landprice(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XPT002", tiles)
    return JSONResponse(content=result)


@app.get("/api/zoning")
async def zoning(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XKT002", tiles)
    return JSONResponse(content=result)


@app.get("/api/liquefaction")
async def liquefaction(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XKT025", tiles)
    return JSONResponse(content=result)


@app.get("/api/flood")
async def flood(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XKT026", tiles)
    return JSONResponse(content=result)


@app.get("/api/steep-slope")
async def steep_slope(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XKT022", tiles)
    return JSONResponse(content=result)


@app.get("/api/schools")
async def schools(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XKT006", tiles)
    return JSONResponse(content=result)


@app.get("/api/medical")
async def medical(lat: float = Query(...), lng: float = Query(...)):
    tiles = get_surrounding_tiles(lat, lng)
    result = await fetch_tile_geojson("XKT010", tiles)
    return JSONResponse(content=result)


LAYER_ENDPOINT_MAP = {
    "landprice": "XPT002",
    "zoning": "XKT002",
    "liquefaction": "XKT025",
    "flood": "XKT026",
    "steep-slope": "XKT022",
    "schools": "XKT006",
    "medical": "XKT010",
}


@app.get("/api/area-data")
async def area_data(
    south: float = Query(...),
    west: float = Query(...),
    north: float = Query(...),
    east: float = Query(...),
    layers: str = Query("landprice,zoning"),
):
    tiles = bbox_to_tiles(south, west, north, east)
    requested_layers = [l.strip() for l in layers.split(",")]

    results: dict[str, dict] = {}
    for layer in requested_layers:
        endpoint = LAYER_ENDPOINT_MAP.get(layer)
        if endpoint is None:
            continue
        results[layer] = await fetch_tile_geojson(endpoint, tiles)

    return JSONResponse(content=results)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)
