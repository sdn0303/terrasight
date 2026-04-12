"""Adapter for 不動産情報ライブラリ CSV data (transaction prices + appraisals)."""
from __future__ import annotations

import csv
import io
import logging
import re
import zipfile
from pathlib import Path

logger = logging.getLogger(__name__)

PROPERTY_TYPE_MAP = {
    "中古マンション等": "condo",
    "宅地(土地と建物)": "land_building",
    "宅地(土地)": "land",
    "林地": "forest",
    "農地": "agriculture",
}

PRICE_CATEGORY_MAP = {
    "不動産取引価格情報": "transaction",
    "成約価格情報": "contract",
}

_PREF_CODE_TO_NAME: dict[str, str] = {
    "01": "北海道", "02": "青森県", "03": "岩手県", "04": "宮城県",
    "05": "秋田県", "06": "山形県", "07": "福島県", "08": "茨城県",
    "09": "栃木県", "10": "群馬県", "11": "埼玉県", "12": "千葉県",
    "13": "東京都", "14": "神奈川県", "15": "新潟県", "16": "富山県",
    "17": "石川県", "18": "福井県", "19": "山梨県", "20": "長野県",
    "21": "岐阜県", "22": "静岡県", "23": "愛知県", "24": "三重県",
    "25": "滋賀県", "26": "京都府", "27": "大阪府", "28": "兵庫県",
    "29": "奈良県", "30": "和歌山県", "31": "鳥取県", "32": "島根県",
    "33": "岡山県", "34": "広島県", "35": "山口県", "36": "徳島県",
    "37": "香川県", "38": "愛媛県", "39": "高知県", "40": "福岡県",
    "41": "佐賀県", "42": "長崎県", "43": "熊本県", "44": "大分県",
    "45": "宮崎県", "46": "鹿児島県", "47": "沖縄県",
}


# ---------------------------------------------------------------------------
# Shared parse helpers
# ---------------------------------------------------------------------------

def _parse_int(raw: str) -> int | None:
    if not raw:
        return None
    raw = raw.strip()
    return int(raw) if raw.isdigit() else None


def _parse_walk_minutes(raw: str) -> int | None:
    if not raw or raw == "":
        return None
    raw = raw.strip()
    if raw.isdigit():
        return int(raw)
    if "30分" in raw and "60分" in raw:
        return 45
    if "1H30" in raw and "2H" in raw:
        return 105
    if "1H" in raw and "1H30" in raw:
        return 75
    if raw.startswith("2H"):
        return 120
    return None


def _parse_building_year(raw: str) -> int | None:
    if not raw:
        return None
    raw = raw.strip()
    if raw == "戦前":
        return 1945
    m = re.search(r"(\d{4})", raw)
    return int(m.group(1)) if m else None


def _parse_area(raw: str) -> int | None:
    if not raw:
        return None
    raw = raw.strip()
    m = re.search(r"(\d+)", raw)
    return int(m.group(1)) if m else None


def _parse_road_width(raw: str) -> float | None:
    if not raw:
        return None
    raw = raw.strip()
    m = re.search(r"([\d.]+)", raw)
    return float(m.group(1)) if m else None


def _parse_quarter(raw: str) -> tuple[int, int] | None:
    m = re.search(r"(\d{4})年第(\d)四半期", raw)
    if m:
        return int(m.group(1)), int(m.group(2))
    return None


# ---------------------------------------------------------------------------
# Transaction prices reader
# ---------------------------------------------------------------------------

def read_transaction_csv(zip_path: Path, pref_code: str) -> list[dict]:
    """Read transaction price CSV from the master ZIP.

    The ZIP contains 47 CSVs named like '13_Tokyo_20053_20253.csv'.
    First row is header (column names in Japanese).
    """
    pref_num = pref_code.lstrip("0") or "0"
    rows: list[dict] = []

    with zipfile.ZipFile(zip_path) as zf:
        target = None
        for name in zf.namelist():
            if name.startswith(f"{pref_num}_") or name.startswith(f"{pref_code}_"):
                target = name
                break
        if target is None:
            logger.warning(f"No CSV for pref_code={pref_code} in {zip_path}")
            return []

        with zf.open(target) as f:
            text = io.TextIOWrapper(f, encoding="cp932", errors="replace")
            reader = csv.reader(text)
            next(reader)  # skip header

            for raw_row in reader:
                if len(raw_row) < 28:
                    continue
                row = _parse_transaction_row(raw_row, pref_code)
                if row is not None:
                    rows.append(row)

    logger.info(f"Read {len(rows)} transaction records for pref={pref_code}")
    return rows


def _parse_transaction_row(raw: list[str], pref_code: str) -> dict | None:
    property_type = PROPERTY_TYPE_MAP.get(raw[0].strip())
    if property_type is None:
        return None

    price_category = PRICE_CATEGORY_MAP.get(raw[1].strip(), "transaction")

    total_price = _parse_int(raw[9])
    if total_price is None or total_price <= 0:
        return None

    quarter = _parse_quarter(raw[27])
    if quarter is None:
        return None

    return {
        "pref_code": pref_code,
        "city_code": raw[3].strip(),
        "city_name": raw[5].strip(),
        "district_name": raw[6].strip() or None,
        "property_type": property_type,
        "price_category": price_category,
        "total_price": total_price,
        "price_per_sqm": _parse_int(raw[13]),
        "area_sqm": _parse_area(raw[12]),
        "floor_plan": raw[11].strip() or None,
        "building_year": _parse_building_year(raw[17]),
        "building_structure": raw[18].strip() or None,
        "current_use": raw[19].strip() or None,
        "city_planning_zone": raw[24].strip() or None,
        "building_coverage": _parse_int(raw[25]),
        "floor_area_ratio": _parse_int(raw[26]),
        "nearest_station": raw[7].strip() or None,
        "station_walk_min": _parse_walk_minutes(raw[8]),
        "front_road_width": _parse_road_width(raw[23]),
        "land_shape": raw[14].strip() or None,
        "transaction_quarter": f"{quarter[0]}Q{quarter[1]}",
        "transaction_year": quarter[0],
        "transaction_q": quarter[1],
    }


# ---------------------------------------------------------------------------
# Appraisal reader
# ---------------------------------------------------------------------------

def read_appraisal_csv(appraisal_dir: Path, pref_code: str) -> list[dict]:
    """Read appraisal (鑑定評価書) TAKUCHI CSV for a single prefecture.

    The directory contains 46 ZIPs named 'hyokasyo-2026-{都道府県名}.zip'.
    Each ZIP has 2026_TAKUCHI_k_{NN}.csv (no header, 1408 columns, CP932).
    """
    pref_name = _PREF_CODE_TO_NAME.get(pref_code)
    if pref_name is None:
        return []

    zip_path = appraisal_dir / f"hyokasyo-2026-{pref_name}.zip"
    if not zip_path.exists():
        logger.warning(f"No appraisal ZIP for {pref_name}: {zip_path}")
        return []

    csv_name = f"2026_TAKUCHI_k_{pref_code}.csv"
    rows: list[dict] = []

    with zipfile.ZipFile(zip_path) as zf:
        if csv_name not in zf.namelist():
            csv_name_alt = f"2026_TAKUCHI_k_{int(pref_code)}.csv"
            if csv_name_alt in zf.namelist():
                csv_name = csv_name_alt
            else:
                logger.warning(f"TAKUCHI CSV not found in {zip_path}")
                return []

        with zf.open(csv_name) as f:
            text = io.TextIOWrapper(f, encoding="cp932", errors="replace")
            reader = csv.reader(text)
            for raw_row in reader:
                if len(raw_row) < 60:
                    continue
                row = _parse_appraisal_row(raw_row, pref_code)
                if row is not None:
                    rows.append(row)

    logger.info(f"Read {len(rows)} appraisal records for pref={pref_code}")
    return rows


def _parse_appraisal_row(raw: list[str], pref_code: str) -> dict | None:
    """Parse appraisal CSV row (1408 columns, no header).

    Key column indices (0-based):
      0: 価格時点(year), 1: 県コード, 2: 市区町村コード(3-digit),
      3: 地域名, 4: 用途区分, 5: 連番, 10: 評価員番号,
      18: 鑑定評価額, 19: 1㎡単価, 26: 所在地番, 27: 住居表示,
      29: 地積, 35: 現況, 42: 道路幅員, 50: 交通施設, 51: 距離(m),
      54: 用途地域コード, 55: 建ぺい率, 56: 容積率,
      102: 比準価格, 103: 収益価格, 104: 積算価格,
      1307: 不動産ID
    """
    survey_year = _parse_int(raw[0])
    if survey_year is None:
        return None

    price = _parse_int(raw[18])
    price_sqm = _parse_int(raw[19])
    if price is None or price <= 0 or price_sqm is None or price_sqm <= 0:
        return None

    city_code_local = raw[2].strip()
    full_city_code = pref_code + city_code_local.zfill(3)

    appraiser_str = raw[10].strip() if len(raw) > 10 else "1"
    appraiser_no = int(appraiser_str) if appraiser_str.isdigit() else 1

    fudosan_id = None
    if len(raw) > 1307:
        fid = raw[1307].strip()
        if len(fid) >= 17 and fid.isdigit():
            fudosan_id = fid

    comparable = _parse_int(raw[102]) if len(raw) > 102 else None
    yield_price = _parse_int(raw[103]) if len(raw) > 103 else None
    cost_price = _parse_int(raw[104]) if len(raw) > 104 else None

    return {
        "pref_code": pref_code,
        "city_code": full_city_code,
        "city_name": raw[3].strip(),
        "land_use_code": raw[4].strip(),
        "sequence_no": _parse_int(raw[5]) or 0,
        "appraiser_no": appraiser_no,
        "survey_year": survey_year,
        "appraisal_price": price,
        "price_per_sqm": price_sqm,
        "address": raw[26].strip(),
        "display_address": raw[27].strip() or None,
        "lot_area_sqm": float(raw[29]) if raw[29].strip() else None,
        "current_use_code": raw[35].strip() or None,
        "zone_code": raw[54].strip() or None,
        "building_coverage": _parse_int(raw[55]),
        "floor_area_ratio": _parse_int(raw[56]),
        "nearest_station": raw[50].strip() or None,
        "station_distance_m": _parse_int(raw[51]),
        "front_road_width": _parse_road_width(raw[42]),
        "fudosan_id": fudosan_id,
        "comparable_price": comparable,
        "yield_price": yield_price,
        "cost_price": cost_price,
    }
