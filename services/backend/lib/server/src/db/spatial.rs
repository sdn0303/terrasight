use sqlx::Postgres;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;

/// Bind bounding-box parameters in the order required by
/// `ST_MakeEnvelope($1, $2, $3, $4, 4326)`.
///
/// PostGIS `ST_MakeEnvelope` expects `(xmin, ymin, xmax, ymax)` which maps to
/// `(west, south, east, north)` for geographic coordinates.
///
/// # Examples
///
/// ```rust,ignore
/// use terrasight_server::db::spatial::bind_bbox;
///
/// let query = sqlx::query_as::<_, (i64,)>(
///     "SELECT id FROM land_prices WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))",
/// );
/// let query = bind_bbox(query, 139.6, 35.5, 139.9, 35.8);
/// ```
pub fn bind_bbox<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
) -> QueryAs<'q, Postgres, O, PgArguments>
where
    O: Send + Unpin,
{
    query.bind(west).bind(south).bind(east).bind(north)
}

/// Bind coordinate parameters in the order required by `ST_MakePoint($1, $2)`.
///
/// PostGIS `ST_MakePoint` takes `(x, y)` which is `(longitude, latitude)` for
/// geographic coordinates per RFC 7946.
///
/// # Examples
///
/// ```rust,ignore
/// use terrasight_server::db::spatial::bind_coord;
///
/// let query = sqlx::query_as::<_, (i64,)>(
///     "SELECT COUNT(*) FROM schools WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)",
/// );
/// let query = bind_coord(query, 139.76, 35.68);
/// ```
pub fn bind_coord<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    lng: f64,
    lat: f64,
) -> QueryAs<'q, Postgres, O, PgArguments>
where
    O: Send + Unpin,
{
    query.bind(lng).bind(lat)
}

#[cfg(test)]
mod tests {
    // bind_bbox and bind_coord are thin wrappers over sqlx's bind() method.
    // The sqlx QueryAs type requires a live database connection to execute,
    // so functional correctness is covered by integration tests.
    // We verify here that the module compiles and the public API is accessible.
    use super::*;

    fn _assert_bbox_fn_exists() {
        // Compile-time check: function signatures are callable with the right types.
        let _: fn(
            QueryAs<'_, Postgres, (i64,), PgArguments>,
            f64,
            f64,
            f64,
            f64,
        ) -> QueryAs<'_, Postgres, (i64,), PgArguments> = bind_bbox::<(i64,)>;
    }

    fn _assert_coord_fn_exists() {
        let _: fn(
            QueryAs<'_, Postgres, (i64,), PgArguments>,
            f64,
            f64,
        ) -> QueryAs<'_, Postgres, (i64,), PgArguments> = bind_coord::<(i64,)>;
    }
}
