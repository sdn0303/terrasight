#!/usr/bin/env bash
# =============================================================================
# download-data.sh — Download all government datasets for the platform
# =============================================================================
# Usage:
#   ./scripts/commands/download-data.sh              # Download all datasets
#   ./scripts/commands/download-data.sh --section X  # Download specific section (1-9)
#   ./scripts/commands/download-data.sh --status      # Show download status only
#
# Data sources:
#   1. N03  — Administrative boundaries (NLNI)
#   2. L01  — Land price public notice (NLNI, 47 prefs, multi-year)
#   3. A29  — Zoning / land use zones (NLNI, 47 prefs)
#   4. A31b — Flood inundation zones (NLNI, mesh-based)
#   5. Hazard datasets: A33 landslide, A40 tsunami, A47 steep slope
#   6. Infrastructure: N02 railway, S12 stations, P04 medical, P29 schools
#   7. New datasets: N07 bus routes, P11 bus stops, L02 pref land survey
#   8. Terrain: landform, geology, soil (Land Classification Survey)
#   9. Special: A16 DID, 500m population mesh, J-SHIS seismic
#
# Manual downloads (cannot be automated):
#   - Tokyo liquefaction PL map: https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/layertable.html
#   - J-SHIS bulk download: https://www.j-shis.bosai.go.jp/download (requires interactive selection)
#   - A55 urban planning: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A55-2024.html (dynamic UI)
#   - PLATEAU 3D models: https://www.geospatial.jp/ckan/dataset/plateau-tokyo23ku
#   - e-Stat census data: https://www.e-stat.go.jp/api/ (requires API key registration)
# =============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA_DIR="$ROOT/data/raw"
NLNI_BASE="https://nlftp.mlit.go.jp/ksj/gml/data"
KOKJO_BASE="https://nlftp.mlit.go.jp/kokjo/inspect/landclassification"

SECTION="${2:-all}"
STATUS_ONLY=false

for arg in "$@"; do
  case "$arg" in
    --status) STATUS_ONLY=true ;;
    --section) ;; # handled via $2
  esac
done

mkdir -p "$DATA_DIR"

# ---------------------------------------------------------------------------
# Helper: download with skip-if-exists
# ---------------------------------------------------------------------------
dl() {
  local url="$1"
  local outfile="$2"
  local label="${3:-}"

  if [ -f "$outfile" ]; then
    return 0
  fi

  if curl -sL --fail -o "$outfile" "$url" 2>/dev/null; then
    local size
    size=$(du -h "$outfile" | cut -f1)
    [ -n "$label" ] && echo "  $label: $size"
    return 0
  else
    rm -f "$outfile"
    return 1
  fi
}

# ---------------------------------------------------------------------------
# Helper: download per-prefecture (codes 01-47)
# ---------------------------------------------------------------------------
dl_per_pref() {
  local url_template="$1"  # must contain {PREF} placeholder
  local out_template="$2"  # must contain {PREF} placeholder
  local label="$3"
  local count=0
  local skipped=0

  for code in $(seq -w 1 47); do
    local pref
    pref=$(printf "%02d" "$code")
    local url="${url_template//\{PREF\}/$pref}"
    local out="${out_template//\{PREF\}/$pref}"

    if [ -f "$out" ]; then
      skipped=$((skipped + 1))
      continue
    fi

    if dl "$url" "$out"; then
      count=$((count + 1))
    fi
  done

  echo "  $label: $count new, $skipped existing"
}

# ---------------------------------------------------------------------------
# Status report
# ---------------------------------------------------------------------------
show_status() {
  echo "=== Data Download Status ==="
  echo ""
  printf "  %-25s %5s files  %s\n" "N03 (boundaries)" "$(ls "$DATA_DIR"/N03-* 2>/dev/null | wc -l | tr -d ' ')" "need: N03-2025"
  printf "  %-25s %5s files  %s\n" "L01 (land price)" "$(ls "$DATA_DIR"/L01-* 2>/dev/null | wc -l | tr -d ' ')" "need: 47 pref x years"
  printf "  %-25s %5s files  %s\n" "L02 (pref land survey)" "$(ls "$DATA_DIR"/L02-* 2>/dev/null | wc -l | tr -d ' ')" "need: 47 pref"
  printf "  %-25s %5s files  %s\n" "A29 (zoning)" "$(ls "$DATA_DIR"/A29-* 2>/dev/null | wc -l | tr -d ' ')" "need: 47 x 2 pref"
  printf "  %-25s %5s files  %s\n" "A31b (flood)" "$(ls "$DATA_DIR"/A31b-* 2>/dev/null | wc -l | tr -d ' ')" "mesh-based"
  printf "  %-25s %5s files  %s\n" "A33 (landslide)" "$(ls "$DATA_DIR"/A33-* 2>/dev/null | wc -l | tr -d ' ')" "nationwide"
  printf "  %-25s %5s files  %s\n" "A40 (tsunami)" "$(ls "$DATA_DIR"/A40-* 2>/dev/null | wc -l | tr -d ' ')" "coastal prefs"
  printf "  %-25s %5s files  %s\n" "A47 (steep slope)" "$(ls "$DATA_DIR"/A47-* 2>/dev/null | wc -l | tr -d ' ')" "47 pref"
  printf "  %-25s %5s files  %s\n" "N02 (railway)" "$(ls "$DATA_DIR"/N02-* 2>/dev/null | wc -l | tr -d ' ')" "latest year"
  printf "  %-25s %5s files  %s\n" "N07 (bus routes)" "$(ls "$DATA_DIR"/N07-* 2>/dev/null | wc -l | tr -d ' ')" "47 pref"
  printf "  %-25s %5s files  %s\n" "S12 (stations)" "$(ls "$DATA_DIR"/S12-* 2>/dev/null | wc -l | tr -d ' ')" "latest year"
  printf "  %-25s %5s files  %s\n" "P04 (medical)" "$(ls "$DATA_DIR"/P04-* 2>/dev/null | wc -l | tr -d ' ')" "47 pref"
  printf "  %-25s %5s files  %s\n" "P11 (bus stops)" "$(ls "$DATA_DIR"/P11-* 2>/dev/null | wc -l | tr -d ' ')" "47 pref"
  printf "  %-25s %5s files  %s\n" "P29 (schools)" "$(ls "$DATA_DIR"/P29-* 2>/dev/null | wc -l | tr -d ' ')" "47 pref"
  printf "  %-25s %5s files  %s\n" "A16 (DID)" "$(ls "$DATA_DIR"/A16-* 2>/dev/null | wc -l | tr -d ' ')" "nationwide"
  printf "  %-25s %5s files  %s\n" "500m pop mesh" "$(ls "$DATA_DIR"/500m* 2>/dev/null | wc -l | tr -d ' ')" "nationwide"
  printf "  %-25s %5s files  %s\n" "Landform/Geology/Soil" "$(ls "$DATA_DIR"/landform* "$DATA_DIR"/geology* "$DATA_DIR"/soil* 2>/dev/null | wc -l | tr -d ' ')" "nationwide"
  echo ""
  echo "  Total: $(ls "$DATA_DIR"/ 2>/dev/null | wc -l | tr -d ' ') files, $(du -sh "$DATA_DIR" | cut -f1)"
  echo ""
  echo "  === Manual download required ==="
  echo "  Tokyo liquefaction PL:  https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/layertable.html"
  echo "  J-SHIS seismic bulk:    https://www.j-shis.bosai.go.jp/download"
  echo "  A55 urban planning:     https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A55-2024.html"
  echo "  PLATEAU 3D models:      https://www.geospatial.jp/ckan/dataset/plateau-tokyo23ku"
  echo "  e-Stat census API:      https://www.e-stat.go.jp/api/"
}

if $STATUS_ONLY; then
  show_status
  exit 0
fi

echo "============================================================"
echo " Government Data Download Pipeline"
echo " Started: $(date '+%Y-%m-%d %H:%M:%S')"
echo " Target:  $DATA_DIR"
echo "============================================================"
echo ""

# ═══════════════════════════════════════════════════════════════
# Section 1: N03 — Administrative Boundaries (2025)
# Source: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2025.html
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "1" ]]; then
  echo "--- [1/9] N03: Administrative Boundaries (2025) ---"
  dl "$NLNI_BASE/N03/N03-2025/N03-20250101_GML.zip" \
     "$DATA_DIR/N03-20250101_GML.zip" \
     "N03-2025 nationwide" || echo "  WARN: N03-2025 download failed"
  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 2: L01 — Land Price Public Notice (multi-year, 47 prefs)
# Source: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-L01-2026.html
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "2" ]]; then
  echo "--- [2/9] L01: Land Prices ---"
  # Latest year (2026) per-prefecture
  for year in 2026 2025 2024; do
    echo "  Year $year:"
    dl_per_pref \
      "$NLNI_BASE/L01/L01-${year}/L01-${year}_{PREF}_GML.zip" \
      "$DATA_DIR/L01-${year}_{PREF}_GML.zip" \
      "L01-${year}"
  done
  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 3: A29 — Zoning / Land Use Zones (47 prefs)
# Source: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A29-v2_1.html
# Note: A29 uses year code "11" (2011 base) per-pref
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "3" ]]; then
  echo "--- [3/9] A29: Zoning (47 prefs) ---"
  # A29 files use pattern: A29-11_{PREF}_GML.zip (year 2011 base)
  # Also try newer versions
  for ver in 11; do
    dl_per_pref \
      "$NLNI_BASE/A29/A29-${ver}/A29-${ver}_{PREF}_GML.zip" \
      "$DATA_DIR/A29-${ver}_{PREF}_GML.zip" \
      "A29-${ver}"
  done
  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 4: A31b — Flood Inundation Zones (mesh-based, 2024)
# Source: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A31b-2024.html
# Mesh-based download — too many files to enumerate, use existing
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "4" ]]; then
  echo "--- [4/9] A31b: Flood Inundation ---"
  existing=$(ls "$DATA_DIR"/A31b-* 2>/dev/null | wc -l | tr -d ' ')
  echo "  $existing files already downloaded (mesh-based, download manually for new areas)"
  echo "  URL: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A31b-2024.html"
  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 5: Hazard — A33 Landslide, A40 Tsunami, A47 Steep Slope
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "5" ]]; then
  echo "--- [5/9] Hazard datasets ---"

  # A33 Landslide Warning Zones (nationwide GeoJSON, 2024)
  echo "  A33 (landslide):"
  dl "$NLNI_BASE/A33/A33-24/A33-24_00_GEOJSON.zip" \
     "$DATA_DIR/A33-24_00_GEOJSON.zip" \
     "A33-24 nationwide" || echo "    already exists or unavailable"

  # A40 Tsunami Inundation (per-prefecture, 2016 base)
  echo "  A40 (tsunami):"
  dl_per_pref \
    "$NLNI_BASE/A40/A40-16/A40-16_{PREF}_GML.zip" \
    "$DATA_DIR/A40-16_{PREF}_GML.zip" \
    "A40-16"

  # A47 Steep Slope (per-prefecture, 2021)
  echo "  A47 (steep slope):"
  dl_per_pref \
    "$NLNI_BASE/A47/A47-21/A47-21_{PREF}_GML.zip" \
    "$DATA_DIR/A47-21_{PREF}_GML.zip" \
    "A47-21"

  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 6: Infrastructure — N02, S12, P04, P29
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "6" ]]; then
  echo "--- [6/9] Infrastructure datasets ---"

  # N02 Railway Lines (nationwide, latest year)
  echo "  N02 (railway lines):"
  dl "$NLNI_BASE/N02/N02-23/N02-23_GML.zip" \
     "$DATA_DIR/N02-23_GML.zip" \
     "N02-23 nationwide" || true

  # S12 Station Ridership (nationwide, latest available)
  echo "  S12 (stations):"
  dl "$NLNI_BASE/S12/S12-22/S12-22_GML.zip" \
     "$DATA_DIR/S12-22_GML.zip" \
     "S12-22 nationwide" || true

  # P04 Medical Facilities (per-prefecture, 2020)
  echo "  P04 (medical):"
  dl_per_pref \
    "$NLNI_BASE/P04/P04-20/P04-20_{PREF}_GML.zip" \
    "$DATA_DIR/P04-20_{PREF}_GML.zip" \
    "P04-20"

  # P29 Schools (per-prefecture, 2021)
  echo "  P29 (schools):"
  dl_per_pref \
    "$NLNI_BASE/P29/P29-21/P29-21_{PREF}_GML.zip" \
    "$DATA_DIR/P29-21_{PREF}_GML.zip" \
    "P29-21"

  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 7: New datasets — N07 Bus Routes, P11 Bus Stops, L02 Pref Land Survey
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "7" ]]; then
  echo "--- [7/9] New datasets ---"

  # N07 Bus Routes (per-prefecture, 2022)
  echo "  N07 (bus routes):"
  dl_per_pref \
    "$NLNI_BASE/N07/N07-22/N07-22_{PREF}_GML.zip" \
    "$DATA_DIR/N07-22_{PREF}_GML.zip" \
    "N07-22"

  # P11 Bus Stops (per-prefecture, 2022)
  echo "  P11 (bus stops):"
  dl_per_pref \
    "$NLNI_BASE/P11/P11-22/P11-22_{PREF}_GML.zip" \
    "$DATA_DIR/P11-22_{PREF}_GML.zip" \
    "P11-22"

  # L02 Prefectural Land Survey (per-prefecture, 2020)
  echo "  L02 (pref land survey):"
  dl_per_pref \
    "$NLNI_BASE/L02/L02-20/L02-20_{PREF}_GML.zip" \
    "$DATA_DIR/L02-20_{PREF}_GML.zip" \
    "L02-20"

  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 8: Terrain — Landform, Geology, Soil (Land Classification Survey)
# Source: https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/download.html
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "8" ]]; then
  echo "--- [8/9] Terrain datasets (Land Classification Survey) ---"

  # 20万分の1 Landform classification (nationwide)
  echo "  Landform (1:200,000):"
  dl "$KOKJO_BASE/download/land/landform.zip" \
     "$DATA_DIR/landform.zip" \
     "landform nationwide" || echo "    already exists or unavailable"

  # 20万分の1 Geology (per-prefecture)
  echo "  Geology (1:200,000):"
  dl "$KOKJO_BASE/download/land/geology.zip" \
     "$DATA_DIR/geology.zip" \
     "geology nationwide" || echo "    already exists or unavailable"

  # 20万分の1 Soil (per-prefecture)
  echo "  Soil (1:200,000):"
  dl "$KOKJO_BASE/download/land/soil.zip" \
     "$DATA_DIR/soil.zip" \
     "soil nationwide" || echo "    already exists or unavailable"

  # Disaster history (土地履歴調査)
  echo "  Disaster history:"
  dl "$KOKJO_BASE/download/land/others.zip" \
     "$DATA_DIR/others.zip" \
     "disaster history" || echo "    already exists or unavailable"

  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Section 9: Special — A16 DID, 500m Pop Mesh, Seismic
# ═══════════════════════════════════════════════════════════════
if [[ "$SECTION" == "all" || "$SECTION" == "9" ]]; then
  echo "--- [9/9] Special datasets ---"

  # A16 Densely Inhabited Districts (nationwide, 2015)
  echo "  A16 (DID):"
  dl "$NLNI_BASE/A16/A16-15/A16-15_GML.zip" \
     "$DATA_DIR/A16-15_GML.zip" \
     "A16-15 nationwide" || echo "    already exists or unavailable"

  # 500m Population Mesh (nationwide, 2024)
  echo "  500m Population Mesh:"
  dl "$NLNI_BASE/mesh500h30/500m_mesh_2024_GEOJSON.zip" \
     "$DATA_DIR/500m_mesh_2024_GEOJSON.zip" \
     "500m mesh 2024" || echo "    already exists or unavailable"

  # L03-b Land Use Fine Mesh (nationwide, 2021)
  echo "  L03-b (land use mesh):"
  dl "$NLNI_BASE/L03-b/L03-b-21/L03-b-21_GEOJSON.zip" \
     "$DATA_DIR/L03-b-21_GEOJSON.zip" \
     "L03-b 2021" || echo "    already exists or unavailable"

  echo ""
fi

# ═══════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════
echo "============================================================"
echo " Download complete: $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================================"
show_status
