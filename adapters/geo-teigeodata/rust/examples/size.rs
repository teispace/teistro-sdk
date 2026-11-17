//! The size probe of `docs/03-design/geo-port.md` §13.3: a provider over
//! packs read into memory, asked every question through `dyn GeoProvider`.
//! Built with `--features probe-baseline`, the same program answers from an
//! empty `GeoTable` instead, so the difference of the two binaries is what
//! the adapter and its engine add to a consumer.
//!
//! ```sh
//! cargo build --profile size --example size --no-default-features --features terrain,zone-rules
//! cargo build --profile size --example size --no-default-features --features probe-baseline
//! ```
#![allow(
    clippy::print_stdout,
    reason = "the probe prints its answers, so that none is optimised away"
)]

use teistro_core::error::Error;
use teistro_port_geo::{
    GeoPoint, GeoProvider, PlaceListing, PlaceNamesRequest, PlaceReverse, PlaceSearch, list, names,
    resolve, reverse, search,
};

/// The provider the probe asks: the adapter over the packs named on the
/// command line, or, for the baseline, an empty table.
fn provider(packs: &[String]) -> Result<Box<dyn GeoProvider>, Error> {
    #[cfg(not(feature = "probe-baseline"))]
    {
        let bytes = packs
            .iter()
            .map(|path| std::fs::read(path).map_err(|why| Error::invalid_arg(why.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Box::new(teistro_geo_teigeodata::TeigeodataGeo::from_bytes(
            bytes,
        )?))
    }
    #[cfg(feature = "probe-baseline")]
    {
        let _ = packs;
        Ok(Box::new(teistro_port_geo::GeoTable::from_json(
            r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#,
        )?))
    }
}

fn main() -> Result<(), Error> {
    let packs: Vec<String> = std::env::args().skip(1).collect();
    let geo = provider(&packs)?;
    let at = GeoPoint::try_from_degrees(27.7172, 85.324)?;
    println!("{:?}", resolve(&*geo, at)?);
    println!("{:?}", geo.time_zones().map(|zones| zones.zones().len()));
    let found = search(
        &*geo,
        &PlaceSearch {
            query: String::from("Kathmandu"),
            focus: None,
            limit: 3,
            lang: Some(String::from("ne")),
        },
    );
    println!("{found:?}");
    println!(
        "{:?}",
        reverse(
            &*geo,
            &PlaceReverse {
                latitude_deg: 27.7172,
                longitude_deg: 85.324,
                limit: 3,
                lang: None
            }
        )?
    );
    println!(
        "{:?}",
        names(
            &*geo,
            &PlaceNamesRequest {
                id: String::from("tgd:np/326"),
                langs: vec![String::from("ne")]
            }
        )?
    );
    let listing = PlaceListing {
        within: String::from("NP"),
        level: Some(1),
        kinds: Vec::new(),
        depth: Some(1),
        limit: 10,
        lang: None,
    };
    println!("{:?}", list(&*geo, &listing)?);
    Ok(())
}
