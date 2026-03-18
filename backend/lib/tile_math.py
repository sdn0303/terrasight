import math


def latlng_to_tile(lat: float, lng: float, zoom: int = 15) -> tuple[int, int]:
    """Convert lat/lng to Slippy Map tile coordinates at given zoom level."""
    n = 2 ** zoom
    x = int((lng + 180.0) / 360.0 * n)
    lat_rad = math.radians(lat)
    y = int((1.0 - math.log(math.tan(lat_rad) + 1.0 / math.cos(lat_rad)) / math.pi) / 2.0 * n)
    return (x, y)


def get_surrounding_tiles(lat: float, lng: float, zoom: int = 15) -> list[tuple[int, int]]:
    """Return center tile + 8 surrounding tiles (9 total)."""
    cx, cy = latlng_to_tile(lat, lng, zoom)
    tiles = []
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            tiles.append((cx + dx, cy + dy))
    return tiles


def bbox_to_tiles(
    south: float, west: float, north: float, east: float, zoom: int = 15
) -> list[tuple[int, int]]:
    """Return all tiles covering a bounding box, max 100 tiles."""
    min_x, max_y = latlng_to_tile(south, west, zoom)
    max_x, min_y = latlng_to_tile(north, east, zoom)
    tiles = []
    for x in range(min_x, max_x + 1):
        for y in range(min_y, max_y + 1):
            tiles.append((x, y))
            if len(tiles) >= 100:
                return tiles
    return tiles
