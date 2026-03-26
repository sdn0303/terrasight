# Government Public Data Sources - Comprehensive Research Report

> Date: 2026-03-26
> Purpose: Catalog all usable government data sources for real estate location analysis, evaluate cross-analysis potential, and identify data gaps.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Data Source Catalog](#2-data-source-catalog)
3. [REINFOLIB API (31 Endpoints)](#3-reinfolib-api)
4. [National Land Numerical Information (NLNI)](#4-national-land-numerical-information)
5. [J-SHIS Seismic Hazard Data](#5-j-shis-seismic-hazard-data)
6. [Land Classification Survey](#6-land-classification-survey)
7. [Tokyo Metropolitan Government Data](#7-tokyo-metropolitan-government-data)
8. [e-Stat / Census Data](#8-e-stat-census-data)
9. [PLATEAU 3D City Models](#9-plateau-3d-city-models)
10. [Cross-Analysis Model Enhancement](#10-cross-analysis-model-enhancement)
11. [Data Gaps & Future Sources](#11-data-gaps-future-sources)
12. [Scraping Script Requirements](#12-scraping-script-requirements)

---

## 1. Executive Summary

### Current State

- **22 datasets / 6.3 GB** currently identified in the project
- **16 datasets ready**, 6 need conversion/API implementation
- **6 of 19 backend layers** returning data (32% operational)

### After This Research

- **60+ datasets** identified across 7 government portals
- **31 REINFOLIB API endpoints** available (price, planning, disaster, facilities, population)
- **PLATEAU 3D building data** with attributes (use, age, floors) for Tokyo 23 wards
- **e-Stat API** for census/housing statistics at mesh level
- Significant opportunity to enhance all 5 TLS axes

### Key Finding

The **REINFOLIB API alone provides 31 endpoints** covering almost all data needed for the 5-axis TLS scoring system. This should be the primary data source, supplemented by NLNI static files and J-SHIS for seismic data.

---

## 2. Data Source Catalog (Master Index)

### By TLS Axis

| Axis | Current Sources | New Sources Identified | Impact |
|------|----------------|----------------------|--------|
| S1 Disaster (25%) | Flood A31b, steep slope A47 | Liquefaction PL, tsunami A40, landslide A33, REINFOLIB disaster (7 APIs), disaster history | HIGH |
| S2 Terrain (15%) | AVS30 (J-SHIS planned) | Landform classification, geology, soil (all scales), elevation mesh G04, large fill A54 | HIGH |
| S3 Livability (25%) | Schools P29, medical P04 | REINFOLIB facilities (8 APIs), bus routes N07, parks, libraries, nurseries, fire stations | MEDIUM |
| S4 Future (15%) | Population mesh (static) | REINFOLIB 250m pop mesh, ridership API, urban planning A55, DID time series, housing vacancy | HIGH |
| S5 Price (20%) | Land price L01, transactions | REINFOLIB price APIs (4), prefectural land survey L02, PLATEAU building attributes | HIGH |

### By Data Portal

| Portal | URL | Datasets | Format | Auth Required |
|--------|-----|----------|--------|---------------|
| REINFOLIB | reinfolib.mlit.go.jp | 31 APIs | PBF, GeoJSON | API Key |
| NLNI (国土数値情報) | nlftp.mlit.go.jp/ksj/ | 40+ | GML, SHP, GeoJSON | None |
| J-SHIS | j-shis.bosai.go.jp | 6 categories | CSV, SHP, KML, API | None |
| Land Survey (国土調査) | nlftp.mlit.go.jp/kokjo/ | 10+ | SHP | None |
| Tokyo Metro | doboku.metro.tokyo.lg.jp | 9 layers | SHP | None |
| e-Stat | e-stat.go.jp | Census+Housing | CSV, API | API Key |
| PLATEAU | geospatial.jp/plateau | 300+ cities | CityGML, GeoJSON | None |

---

## 3. REINFOLIB API (31 Endpoints)

Source: https://www.reinfolib.mlit.go.jp/help/apiManual/
Auth: `Ocp-Apim-Subscription-Key` header
Format: PBF (vector tiles) or GeoJSON
Status: **API key already configured** in project (health endpoint confirms `reinfolib_key_set: true`)

### 3.1 Price Information (4 APIs)

| ID | Name | Data | Period | TLS Axis |
|----|------|------|--------|----------|
| XIT001 | Transaction Price API | Real estate transaction prices + REINS contract prices | 2005Q3~ / 2021Q1~ | S5 |
| XCT001 | Appraisal Report API | Land price public notices (last 5 years) | 5yr | S5 |
| XPT001 | Transaction Price Points API | Geographic points for map display | Current | S5 |
| XPT002 | Land Price Points API | Public land prices (1995~), prefectural survey (1997~) | 30yr | S5 |

**Key for S5 Price Analysis:**
- XIT001 provides REINS-origin contract prices (not just asking prices)
- XPT002 gives 30-year time series for CAGR calculation
- Transaction volume (V_vol) calculable from XIT001 density

### 3.2 Urban Planning (7 APIs)

| ID | Name | Data | Year | TLS Axis |
|----|------|------|------|----------|
| XKT001 | Urban Planning Area API | Urban planning zones, area classification | R6 | S4 |
| XKT002 | Zoning API | Land use zones (FAR, BCR) | R6 | S4, S5 |
| XKT003 | Location Optimization Plan API | Compact city zones | R6 | S4 |
| XKT014 | Fire Prevention Zone API | Fire/semi-fire prevention zones | R6 | S1 |
| XKT023 | District Plan API | District plans | R6 | S4 |
| XKT024 | High-rise Use Zone API | High-density use zones | R6 | S4 |
| XKT030 | Urban Planning Road API | Planned roads | R6 | S4 |

**Key for S4 Future Potential:**
- XKT003 (Location optimization) = strong future potential signal
- XKT024 (High-rise zones) + XKT002 (FAR capacity) = development upside
- XKT030 (Planned roads) = infrastructure investment signal

### 3.3 Surrounding Facilities (8 APIs)

| ID | Name | Facilities | Year | TLS Axis |
|----|------|-----------|------|----------|
| XKT004 | Elementary School District API | School districts | R5 | S3 |
| XKT005 | Junior High School District API | School districts | R5 | S3 |
| XKT006 | School API | Schools (all types) | R5 | S3 |
| XKT007 | Nursery/Kindergarten API | Nurseries, kindergartens, welfare | R5 | S3 |
| XKT010 | Medical Facility API | Hospitals, clinics | R2 | S3 |
| XKT011 | Welfare Facility API | Welfare facilities | R5 | S3 |
| XKT017 | Library API | Public libraries | H25 | S3 |
| XKT018 | Municipal Office API | City halls, public meeting facilities | R4 | S3 |

**Key for S3 Livability:**
- Currently using only schools + medical
- Adding nursery/kindergarten (XKT007) = family livability
- Library + welfare = elderly/cultural livability
- School district data (XKT004/005) = education quality proxy

### 3.4 Disaster/Hazard (8 APIs)

| ID | Name | Hazard Type | Year | TLS Axis |
|----|------|------------|------|----------|
| XKT016 | Disaster Hazard Zone API | Designated hazard zones | R3 | S1 |
| XKT021 | Landslide Prevention API | Landslide prevention zones | R3 | S1 |
| XKT022 | Steep Slope Collapse API | Steep slope danger zones | R3 | S1 |
| XKT025 | Liquefaction Tendency API | Liquefaction risk by landform | Latest | S1, S2 |
| XKT026 | Flood Inundation API | Flood inundation zones | R6 | S1 |
| XKT027 | Storm Surge Inundation API | Storm surge zones | R6 | S1 |
| XKT028 | Tsunami Inundation API | Tsunami inundation zones | R6 | S1 |
| XKT029 | Sediment Disaster API | Sediment disaster warning zones | R6 | S1 |

**Key for S1 Disaster Risk:**
- **XKT025 Liquefaction** = critical missing piece, available via API!
- XKT027 Storm Surge = new hazard type not in current model
- All R6 year data = very recent

### 3.5 Population & Infrastructure (5 APIs)

| ID | Name | Data | Year | TLS Axis |
|----|------|------|------|----------|
| XKT013 | Future Population Mesh API | 250m mesh population projections | Latest | S4 |
| XKT015 | Station Ridership API | Ridership by station | R5 | S3, S4 |
| XKT031 | DID API | Densely Inhabited District | 2020 | S4 |
| XKT020 | Large Fill Map API | Large-scale embankment sites | R5 | S1, S2 |
| XKT019 | Natural Park API | Natural park zones | H27 | S3 |

**Key:**
- XKT013 at **250m mesh** (vs current 500m) = 4x resolution improvement
- XKT015 ridership = key for L_transit sub-score
- XKT020 large fill = ground stability risk factor

### 3.6 Other (3 APIs)

| ID | Name | Data |
|----|------|------|
| XIT002 | Municipality List API | Administrative codes |
| XGT001 | Emergency Shelter API | Designated evacuation sites (R8) |
| XST001 | Disaster History API | Historical disaster records |

---

## 4. National Land Numerical Information (NLNI)

Source: https://nlftp.mlit.go.jp/ksj/gml/gml_datalist.html
Format: GML, SHP, GeoJSON (all CC BY 4.0 unless noted)

### 4.1 Administrative & Planning

| Code | Name | Latest | Format | Notes |
|------|------|--------|--------|-------|
| **N03** | Administrative Boundaries | **2025** | GeoJSON/SHP/GML | 47 prefectures + all municipalities. **603MB full dataset.** Essential for boundary display requirement. |
| A29 | Zoning (Land Use Zones) | 2019 | GeoJSON | FAR, BCR attributes |
| A50 | Location Optimization Plans | 2020 | GeoJSON | Compact city zones |
| A55 | Urban Planning Decisions | 2024 | GeoJSON | Comprehensive planning data |
| A54 | Large-scale Embankment | 2023 | GeoJSON | Ground risk |

### 4.2 Land Price & Land Use

| Code | Name | Latest | Period | Notes |
|------|------|--------|--------|-------|
| **L01** | Land Price Public Notice | **2026** | 1983-2026 (44yr!) | Core price dataset. Attributes: price/sqm, change rate, land area, zoning, FAR/BCR, nearest station distance, road width, building structure |
| L02 | Prefectural Land Survey | 2025 | Multi-year | Complementary to L01 |
| L03-a | Land Use 3rd Mesh | 2021 | - | Land use classification |
| L03-b | Land Use Fine Mesh | 2021 | - | Detailed land use |

### 4.3 Disaster & Hazard

| Code | Name | Latest | Notes |
|------|------|--------|-------|
| A31a | Flood Inundation (Planning) | 2024 | Design-level flood zones |
| **A31b** | Flood Inundation (Max) | **2024** | Maximum scenario. Currently in use. |
| A33 | Landslide Warning | - | Sediment disaster zones |
| A40 | Tsunami Inundation | 2024 | Prefecture-level |
| A47 | Steep Slope | - | Currently in use |
| A16 | DID (Densely Inhabited) | 2015 | 12 time points (1960-2015) |

### 4.4 Transportation

| Code | Name | Latest | Notes |
|------|------|--------|-------|
| N02 | Railway Lines | Latest | Line geometry + operator |
| S12 | Station Ridership | Multi-year | 13-year time series |
| **N07** | Bus Routes | **2022** | Route lines, operator, frequency, classification (private/community/demand) |
| P11 | Bus Stops | Latest | Stop point locations |
| N08 | Airports (Time Series) | 2011 | Runway, operations |

### 4.5 Facilities

| Code | Name | Latest | Notes |
|------|------|--------|-------|
| P29 | Schools | Latest | Currently in use |
| P04 | Medical Facilities | Latest | Currently in use |
| P14 | Welfare Facilities | 2021 | Nursing homes, daycare |
| P02 | Public Facilities | 2006 | Government offices, libraries |
| P05 | Municipal Offices | Latest | City halls, public halls |

### 4.6 Terrain & Elevation

| Code | Name | Latest | Notes |
|------|------|--------|-------|
| G04 | Elevation/Slope Mesh | Latest | 3rd-5th mesh DEM |
| G08 | Low-lying Areas | 2015 | Below sea level zones |

---

## 5. J-SHIS Seismic Hazard Data

Source: https://www.j-shis.bosai.go.jp/download
API: https://www.j-shis.bosai.go.jp/map/api/

### 5.1 Available Datasets

| Category | Data | Format | Notes |
|----------|------|--------|-------|
| **Probabilistic Seismic Hazard Map** | 30yr/50yr exceedance probability | CSV, SHP, KML | Core for F_seis sub-score |
| Conditional Exceedance Probability | Given-scenario probability | CSV, SHP | |
| **Source Fault Prediction** | Fault-specific ground motion | CSV, SHP | Fault parameters, waveform data |
| **Shallow Subsurface** | AVS30, amplification factor | CSV, SHP, KML | Core for G_avs sub-score |
| **Deep Subsurface** | Deep geological structure | CSV, SHP | Enhanced terrain analysis |
| Affected Population | Estimated casualties | CSV | Risk quantification |

### 5.2 API Endpoints

```
# Surface geology (AVS30)
GET /map/api/sstrct/V3/meshinfo.geojson?position={lng},{lat}&epsg=4326

# Probabilistic seismic hazard
GET /map/api/pshm/V3/meshinfo.geojson?position={lng},{lat}&epsg=4326
```

**Citation requirement:** Must reference specified publications when using data.

---

## 6. Land Classification Survey (国土調査)

Source: https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/download.html

### 6.1 Available Scales

| Scale | Coverage | Data Types | Format |
|-------|----------|-----------|--------|
| 1:500,000 | Nationwide | Landform, geology, soil, volcanic, fault | SHP (24MB total) |
| **1:200,000** | **All 47 prefectures** | Landform, geology, soil | SHP (per-prefecture) |
| 1:50,000 | Major cities | Surface geology, soil | SHP |
| Land History Survey | Major metro areas | Historical disaster, water, land use changes | SHP |

### 6.2 Landform Classification

Source: https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/land/chikei_bunrui.html

- Nationwide GIS data: `landform.zip` (all prefectures) or `{01-47}.zip` (per-prefecture)
- Categories: Plains, terraces, plateaus, mountains, valleys, filled land, reclaimed land
- **Critical for S2 G_form sub-score** (currently not implemented)

### 6.3 Disaster History

Source: https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/land/saigai_rireki.html

- Historical records: floods (pre-1965, post-1975), earthquakes (named events), landslides, storm surge, tsunami, land subsidence
- **Covers named historical events**: 1923 Kanto, 1944 Tonankai, 1945 Mikawa, etc.
- GIS format per metro area
- **Critical for cross-analysis**: historical disaster frequency = forward-looking risk proxy

### 6.4 Seamless Land Conservation Map (20万分の1)

- All 47 prefectures covered
- Integrated landform + hazard assessment
- Quaternary volcanic data separately available

---

## 7. Tokyo Metropolitan Government Data

Source: https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/layertable.html

### Available Layers

| Layer | Format | License | Notes |
|-------|--------|---------|-------|
| **Borehole Logs** | SHP | CC BY 2.1 JP | Ground survey data |
| Soil Test Results | SHP | CC BY 2.1 JP | Lab test data |
| **PL Distribution Map** | SHP | CC BY 2.1 JP | Liquefaction prediction (PL values) |
| **Groundwater Level Map** | SHP | CC BY 2.1 JP | Water table depth |
| **Liquefaction History (1923 Kanto)** | SHP | CC BY 2.1 JP | Historical liquefaction damage |
| **Liquefaction History (2011 Tohoku)** | SHP | CC BY 2.1 JP | Recent liquefaction damage |
| Land Condition Map | SHP | CC BY 2.1 JP | Micro-geomorphology |
| Water Area Transition Maps | SHP | CC BY 2.1 JP | Meiji/Taisho/Showa era changes |
| Wetland/Paddy Distribution | SHP | CC BY 2.1 JP | Historical land use |
| **Fill Material Classification** | SHP | CC BY 2.1 JP | Coastal reclaimed land soil types |

**Key insight:** Tokyo Metro data provides the **most granular liquefaction data** available, with both predictive (PL values) and historical (2 events) data. This is Tokyo-specific but covers the primary target area.

---

## 8. e-Stat / Census Data

Source: https://www.e-stat.go.jp/api/
Auth: User registration + API key

### 8.1 Relevant Statistical Surveys

| Survey | Data | Granularity | Latest |
|--------|------|-------------|--------|
| **National Census** | Population, households, age distribution | 250m/500m/1km mesh, municipality | 2020 |
| **Housing & Land Survey** | Vacancy rate, building age, structure, ownership | Municipality | 2023 (R5) |
| Retail Census | Commercial activity, store count | Municipality, mesh | 2021 |
| Economic Census | Business count, employees | Municipality | 2021 |

### 8.2 Housing & Land Survey (R5)

- **Vacancy rate: 13.8% nationally** (2023)
- Available at: prefecture, city, ward, and towns with pop > 15,000
- Attributes: vacancy type (4 categories), building condition, structure
- **Key for S4/S5:** High vacancy = weak demand signal; low vacancy = strong demand

### 8.3 e-Stat API

```
# API base
GET https://api.e-stat.go.jp/rest/3.0/

# Functions:
# - getStatsList: search available statistics
# - getStatsData: retrieve statistical data
# - getMesh: retrieve mesh-level data (250m/500m/1km)
```

---

## 9. PLATEAU 3D City Models

Source: https://www.mlit.go.jp/plateau/open-data/
Download: https://www.geospatial.jp/ckan/dataset/plateau-tokyo23ku

### 9.1 Tokyo 23 Wards Data

- **Area:** 627.57 km2 (all 23 wards)
- **LOD1:** All buildings (box models with height)
- **LOD2:** Detailed models in Ikebukuro, Shinjuku, Shibuya, etc.
- **Format:** CityGML 2.0 (i-UR 1.4 ADE)
- **Years:** 2020, 2022
- **License:** Commercial use OK

### 9.2 Building Attributes (when available)

| Attribute | Description | TLS Relevance |
|-----------|-------------|---------------|
| Building use | Residential/commercial/industrial | S3, S5 |
| Construction year | Building age | S5 (area maturity) |
| Number of floors | Above/below ground | S5 (density indicator) |
| Building area | Footprint size | S5 |
| Total floor area | Gross floor area | S5 |
| Structure type | RC/S/W/SRC | S1 (fire risk), S5 |
| Measured height | Actual building height | Density analysis |

### 9.3 Coverage (as of 2025)

- **~300 cities nationwide** (expanding)
- Attribute richness varies by municipality
- Tokyo 23 wards: most comprehensive

**Key for cross-analysis:** Building density + age distribution = neighborhood maturity signal. Combined with vacancy rate from e-Stat = comprehensive demand analysis.

---

## 10. Cross-Analysis Model Enhancement

### 10.1 Current Model (3 patterns)

```
Value Discovery  = S1 x (100 - V_rel) / 100    -- safe but cheap
Demand Signal    = S3 x S4 / 100                 -- convenient + growing
Ground Safety    = S1 x S2 / 100                  -- disaster x terrain
```

### 10.2 Proposed New Cross-Analysis Patterns

#### Pattern 4: Family Suitability Index
```
Family = (school_district_quality x 0.3 + nursery_access x 0.2
         + park_area x 0.2 + S1 x 0.3)
```
**New data:** XKT004/005 school districts, XKT007 nurseries, park data

#### Pattern 5: Infrastructure Investment Signal
```
Investment_Signal = (urban_plan_road x 0.3 + location_optimization x 0.3
                    + station_ridership_trend x 0.2 + pop_trend x 0.2)
```
**New data:** XKT030 planned roads, XKT003 compact city, XKT015 ridership

#### Pattern 6: Disaster Resilience Score
```
Resilience = S1 x S2 x (1 - disaster_history_frequency x 0.1)
             x shelter_proximity
```
**New data:** XST001 disaster history, XGT001 emergency shelters

#### Pattern 7: Rental Demand Potential
```
Rental = (station_proximity x 0.3 + pop_density x 0.2
         + commercial_density x 0.2 + vacancy_inverse x 0.3)
```
**New data:** e-Stat vacancy rate, PLATEAU building density, ridership

#### Pattern 8: Aging Risk
```
Aging_Risk = elderly_ratio x (1 - medical_access) x (1 - welfare_access)
```
**New data:** Census age mesh, XKT010 medical, XKT011 welfare

### 10.3 S1 Enhancement: Storm Surge (New Sub-Score)

Currently missing: **storm surge (F_surge)**

```
# Add to S1 composition:
F_surge mapping:
  Outside zone = 100
  <0.5m = 80
  0.5-1m = 60
  1-3m = 30
  >3m = 10

# Revised S1:
S1 = min(F_flood, F_liq, F_seis, F_tsun, F_land, F_surge)
     x (0.25xF_flood + 0.20xF_liq + 0.20xF_seis + 0.10xF_tsun
        + 0.10xF_land + 0.10xF_surge + 0.05xF_fill) / 100
```
**Data:** XKT027 storm surge, XKT020 large fill

### 10.4 S2 Enhancement: Full Terrain Scoring

Currently: S2 = G_avs only (Phase 1)

```
# Full implementation:
S2 = 0.40 x G_avs + 0.25 x G_form + 0.20 x G_geo + 0.15 x G_elev

G_form: landform classification mapping
  Plateau/Terrace = 100
  Alluvial fan = 75
  Natural levee = 60
  Alluvial plain = 45
  Reclaimed land = 25
  Fill land = 15

G_geo: geology mapping
  Bedrock = 100
  Gravel = 80
  Sand = 50
  Clay = 30
  Peat = 10

G_elev: elevation/slope
  Flat (<5°) & elevated (>20m) = 100
  Moderate slope (5-15°) = 70
  Steep (>15°) or low-lying (<2m) = 30
```
**Data:** Land classification survey (all scales), G04 elevation mesh

### 10.5 S3 Enhancement: Expanded Livability

Currently: S3 = transit + education + medical

```
# Enhanced S3:
S3 = 0.35 x L_transit + 0.20 x L_edu + 0.20 x L_med
     + 0.10 x L_childcare + 0.10 x L_daily + 0.05 x L_culture

L_childcare: nursery/kindergarten access (XKT007)
L_daily: bus access (N07) + municipal services (XKT018)
L_culture: libraries (XKT017) + parks (XKT019)
```

### 10.6 S4 Enhancement: Comprehensive Future Scoring

Currently: S4 = pop_trend + price_cagr + FAR_capacity

```
# Enhanced S4:
S4 = 0.25 x P_pop + 0.20 x P_price + 0.15 x P_far
     + 0.15 x P_infra + 0.15 x P_vacancy + 0.10 x P_compact

P_infra: planned infrastructure (XKT030 + ridership trend)
P_vacancy: inverse vacancy rate (e-Stat housing survey)
P_compact: location optimization zone inclusion (XKT003)
```

---

## 11. Data Gaps & Future Sources

### 11.1 Still Missing (No Government Source Found)

| Data | Importance | Alternative |
|------|-----------|-------------|
| Crime statistics | Medium | Police agency publishes prefectural reports (PDF scraping) |
| Noise levels | Low | Can approximate from road/rail proximity |
| Air quality | Low | Ministry of Environment monitoring stations (API available) |
| Commercial facilities | Medium | Retail census from e-Stat provides some coverage |
| School quality/ratings | Medium | Not publicly available as data |
| Internet connectivity | Low | NTT/KDDI coverage maps (not open data) |

### 11.2 Potential Future Additions

| Source | Data | Status |
|--------|------|--------|
| Ministry of Environment | Air quality monitoring | API available |
| National Police Agency | Crime statistics by prefecture | PDF reports, needs scraping |
| MLIT Road Bureau | Road traffic census | Available on NLNI |
| Fire & Disaster Mgmt Agency | Fire risk maps | Some prefectures publish |
| Japan Meteorological Agency | Climate data (rainfall, temperature) | API available |

---

## 12. Scraping Script Requirements

### 12.1 Scripts Needed (scripts/ directory)

| Script | Source | Format | Priority |
|--------|--------|--------|----------|
| `download-nlni.sh` | nlftp.mlit.go.jp | ZIP -> GeoJSON/SHP | HIGH |
| `download-n03-boundaries.sh` | nlftp.mlit.go.jp/ksj | N03 GeoJSON (47 prefectures) | HIGH |
| `download-l01-landprice.sh` | nlftp.mlit.go.jp/ksj | L01 2026 GeoJSON | HIGH |
| `download-jshis.sh` | j-shis.bosai.go.jp | CSV/SHP surface geology | HIGH |
| `download-land-survey.sh` | nlftp.mlit.go.jp/kokjo | Landform/geology/soil SHP | MEDIUM |
| `download-tokyo-liq.sh` | doboku.metro.tokyo.lg.jp | PL distribution SHP | HIGH |
| `download-plateau.sh` | geospatial.jp | Tokyo 23ku CityGML | MEDIUM |
| `convert-gml-to-geojson.py` | Local | GML -> GeoJSON conversion | HIGH |
| `import-shp-to-postgis.py` | Local | SHP -> PostGIS import | HIGH |
| `fetch-reinfolib-tiles.py` | reinfolib.mlit.go.jp | API -> GeoJSON cache | MEDIUM |
| `fetch-estat-census.py` | e-stat.go.jp | API -> CSV/JSON | MEDIUM |

### 12.2 Download URL Templates

```bash
# N03 Administrative Boundaries (2025, GeoJSON, nationwide)
https://nlftp.mlit.go.jp/ksj/gml/data/N03/N03-2025/N03-20250101_GML.zip

# L01 Land Prices (2026, GeoJSON, per-prefecture)
https://nlftp.mlit.go.jp/ksj/gml/data/L01/L01-2026/L01-2026_{pref_code}.zip

# A31b Flood (2024, GeoJSON)
https://nlftp.mlit.go.jp/ksj/gml/data/A31b/A31b-2024/A31b-2024_{mesh_code}.zip

# Landform classification (per-prefecture SHP)
https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/download/land/{pref_code}.zip

# Tokyo liquefaction data
https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/download/{layer_name}.zip
```

### 12.3 Data Pipeline Overview

```
scripts/download-*.sh    -> data/raw/{source}/
scripts/convert-*.py     -> data/processed/{format}/
scripts/import-*.py      -> PostgreSQL + PostGIS
                            (or) services/frontend/public/geojson/
```

---

## Appendix: Source URLs

### User-Provided URLs
- https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/layertable.html
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-v3_1.html
- https://www.j-shis.bosai.go.jp/download
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A31b-2024.html
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2025.html
- https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/download.html
- https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/land/chikei_bunrui.html
- https://nlftp.mlit.go.jp/kokjo/inspect/landclassification/land/saigai_rireki.html
- https://nlftp.mlit.go.jp/kokjo/inspect/inspect.html
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A16-v2_3.html
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N08-2021.html (Actually airport data N08, not bus)
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-L01-2026.html
- https://nlftp.mlit.go.jp/ksj/gml/gml_datalist.html

### Additionally Discovered
- https://www.reinfolib.mlit.go.jp/help/apiManual/ (REINFOLIB 31 APIs)
- https://www.e-stat.go.jp/api/ (e-Stat census API)
- https://www.stat.go.jp/data/jyutaku/index.html (Housing & Land Survey)
- https://www.mlit.go.jp/plateau/open-data/ (PLATEAU 3D models)
- https://www.geospatial.jp/ckan/dataset/plateau-tokyo23ku (Tokyo 23ku PLATEAU)
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N07-2022.html (Bus routes N07)
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P11.html (Bus stops P11)
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P14.html (Welfare facilities P14)
- https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A55-2022.html (Urban planning decisions)
- https://www.mlit.go.jp/toshi/tosiko/toshi_tosiko_tk_000087.html (Urban planning GIS national)
- https://www.land.mlit.go.jp/webland/download.html (Transaction price CSV download)
