//! Spatial query helpers for PostGIS parameter binding.
//!
//! Wraps the repetitive `.bind()` chains required by
//! `ST_MakeEnvelope($1,$2,$3,$4,4326)` and `ST_MakePoint($1,$2)` so that
//! call sites pass typed structs ([`GeoBBox`], [`GeoCoord`]) instead of
//! positional `f64` arguments, eliminating parameter-ordering bugs.

use sqlx::Postgres;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use terrasight_geo::{GeoBBox, GeoCoord};

/// Bind bounding-box parameters in the order required by
/// `ST_MakeEnvelope($1, $2, $3, $4, 4326)`.
///
/// PostGIS `ST_MakeEnvelope` expects `(xmin, ymin, xmax, ymax)` which maps to
/// `(west, south, east, north)` for geographic coordinates.
///
/// Using [`GeoBBox`] instead of four positional `f64` arguments prevents
/// silent parameter-ordering bugs at the call site.
///
/// # Examples
///
/// ```rust,ignore
/// use terrasight_geo::GeoBBox;
/// use terrasight_server::db::spatial::bind_bbox;
///
/// let bbox = GeoBBox { south: 35.5, west: 139.6, north: 35.8, east: 139.9 };
/// let query = sqlx::query_as::<_, (i64,)>(
///     "SELECT id FROM land_prices WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))",
/// );
/// let query = bind_bbox(query, &bbox);
/// ```
pub fn bind_bbox<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    bbox: &GeoBBox,
) -> QueryAs<'q, Postgres, O, PgArguments>
where
    O: Send + Unpin,
{
    query
        .bind(bbox.west)
        .bind(bbox.south)
        .bind(bbox.east)
        .bind(bbox.north)
}

/// Bind coordinate parameters in the order required by `ST_MakePoint($1, $2)`.
///
/// PostGIS `ST_MakePoint` takes `(x, y)` which is `(longitude, latitude)` for
/// geographic coordinates per RFC 7946.
///
/// Using [`GeoCoord`] instead of two positional `f64` arguments prevents
/// silent `(lat, lng)` / `(lng, lat)` transposition bugs.
///
/// # Examples
///
/// ```rust,ignore
/// use terrasight_geo::GeoCoord;
/// use terrasight_server::db::spatial::bind_coord;
///
/// let coord = GeoCoord { lng: 139.76, lat: 35.68 };
/// let query = sqlx::query_as::<_, (i64,)>(
///     "SELECT COUNT(*) FROM schools WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)",
/// );
/// let query = bind_coord(query, &coord);
/// ```
pub fn bind_coord<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    coord: &GeoCoord,
) -> QueryAs<'q, Postgres, O, PgArguments>
where
    O: Send + Unpin,
{
    query.bind(coord.lng).bind(coord.lat)
}

#[cfg(test)]
mod tests {
    // bind_bbox and bind_coord are thin wrappers over sqlx's bind() method.
    // The sqlx QueryAs type requires a live database connection to execute,
    // so functional correctness is covered by integration tests.
    // We verify here that the module compiles and the public API is accessible
    // with the new typed-struct signatures.
    use super::*;

    fn _assert_bbox_fn_exists() {
        // Compile-time check: function signatures are callable with the right types.
        // HRTB `for<'a>` is required because both the QueryAs lifetime and the
        // &GeoBBox reference lifetime must be named when coercing to a fn pointer.
        let _: for<'a> fn(
            QueryAs<'a, Postgres, (i64,), PgArguments>,
            &'a GeoBBox,
        ) -> QueryAs<'a, Postgres, (i64,), PgArguments> = bind_bbox::<(i64,)>;
    }

    fn _assert_coord_fn_exists() {
        let _: for<'a> fn(
            QueryAs<'a, Postgres, (i64,), PgArguments>,
            &'a GeoCoord,
        ) -> QueryAs<'a, Postgres, (i64,), PgArguments> = bind_coord::<(i64,)>;
    }
}
