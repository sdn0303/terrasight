import json
import time
from pathlib import Path

import aiosqlite

DB_PATH = Path(__file__).resolve().parent.parent / "data" / "cache.db"


async def init_cache() -> None:
    """Create the cache database and table if they don't exist."""
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    async with aiosqlite.connect(DB_PATH) as db:
        await db.execute(
            """
            CREATE TABLE IF NOT EXISTS api_cache (
                cache_key TEXT PRIMARY KEY,
                response_json TEXT,
                cached_at REAL
            )
            """
        )
        await db.commit()


async def get_cached(key: str, ttl_seconds: int = 86400) -> dict | None:
    """Return cached data if within TTL, else None."""
    async with aiosqlite.connect(DB_PATH) as db:
        cursor = await db.execute(
            "SELECT response_json, cached_at FROM api_cache WHERE cache_key = ?",
            (key,),
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        response_json, cached_at = row
        if time.time() - cached_at > ttl_seconds:
            return None
        return json.loads(response_json)


async def set_cached(key: str, data: dict) -> None:
    """Upsert a cache entry."""
    async with aiosqlite.connect(DB_PATH) as db:
        await db.execute(
            """
            INSERT INTO api_cache (cache_key, response_json, cached_at)
            VALUES (?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                response_json = excluded.response_json,
                cached_at = excluded.cached_at
            """,
            (key, json.dumps(data), time.time()),
        )
        await db.commit()
