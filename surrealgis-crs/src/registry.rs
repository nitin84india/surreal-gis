//! EPSG registry for spatial reference system definitions.
//!
//! Provides proj4 string lookups, geographic CRS classification, and
//! enumeration of known SRIDs for the most commonly used coordinate
//! reference systems.

/// Returns the proj4 definition string for a given SRID, or None if unknown.
pub fn get_proj4_string(srid: i32) -> Option<&'static str> {
    match srid {
        // Geographic CRS
        4326 => Some("+proj=longlat +datum=WGS84 +no_defs +type=crs"),
        4269 => Some("+proj=longlat +datum=NAD83 +no_defs +type=crs"),
        4267 => Some("+proj=longlat +datum=NAD27 +no_defs +type=crs"),
        4258 => Some("+proj=longlat +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +no_defs +type=crs"),
        4148 => Some("+proj=longlat +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +no_defs +type=crs"),
        4674 => Some("+proj=longlat +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +no_defs +type=crs"),
        4283 => Some("+proj=longlat +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +no_defs +type=crs"),
        4612 => Some("+proj=longlat +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +no_defs +type=crs"),
        4490 => Some("+proj=longlat +ellps=GRS80 +no_defs +type=crs"),

        // Web Mercator
        3857 => Some("+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 +units=m +nadgrids=@null +no_defs +type=crs"),

        // World Mercator
        3395 => Some("+proj=merc +lon_0=0 +k=1 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs +type=crs"),

        // ETRS89 / LAEA Europe
        3035 => Some("+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs +type=crs"),

        // RGF93 / Lambert-93 (France)
        2154 => Some("+proj=lcc +lat_0=46.5 +lon_0=3 +lat_1=49 +lat_2=44 +x_0=700000 +y_0=6600000 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs +type=crs"),

        // OSGB36 / British National Grid
        27700 => Some("+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +nadgrids=OSTN15_NTv2_OSGBtoETRS.gsb +units=m +no_defs +type=crs"),

        // US National Atlas Equal Area
        2163 => Some("+proj=laea +lat_0=45 +lon_0=-100 +x_0=0 +y_0=0 +a=6370997 +b=6370997 +units=m +no_defs +type=crs"),

        // NSIDC EASE-Grid North
        3408 => Some("+proj=cea +lon_0=0 +lat_ts=30 +x_0=0 +y_0=0 +a=6371228 +b=6371228 +units=m +no_defs +type=crs"),

        // NSIDC EASE-Grid South
        3409 => Some("+proj=cea +lon_0=0 +lat_ts=30 +x_0=0 +y_0=0 +a=6371228 +b=6371228 +units=m +no_defs +type=crs"),

        // NSIDC EASE-Grid Global
        3410 => Some("+proj=cea +lon_0=0 +lat_ts=30 +x_0=0 +y_0=0 +a=6371228 +b=6371228 +units=m +no_defs +type=crs"),

        // UTM Zones North (WGS 84) — 32601-32660
        32601 => Some("+proj=utm +zone=1 +datum=WGS84 +units=m +no_defs +type=crs"),
        32602 => Some("+proj=utm +zone=2 +datum=WGS84 +units=m +no_defs +type=crs"),
        32603 => Some("+proj=utm +zone=3 +datum=WGS84 +units=m +no_defs +type=crs"),
        32604 => Some("+proj=utm +zone=4 +datum=WGS84 +units=m +no_defs +type=crs"),
        32605 => Some("+proj=utm +zone=5 +datum=WGS84 +units=m +no_defs +type=crs"),
        32606 => Some("+proj=utm +zone=6 +datum=WGS84 +units=m +no_defs +type=crs"),
        32607 => Some("+proj=utm +zone=7 +datum=WGS84 +units=m +no_defs +type=crs"),
        32608 => Some("+proj=utm +zone=8 +datum=WGS84 +units=m +no_defs +type=crs"),
        32609 => Some("+proj=utm +zone=9 +datum=WGS84 +units=m +no_defs +type=crs"),
        32610 => Some("+proj=utm +zone=10 +datum=WGS84 +units=m +no_defs +type=crs"),
        32611 => Some("+proj=utm +zone=11 +datum=WGS84 +units=m +no_defs +type=crs"),
        32612 => Some("+proj=utm +zone=12 +datum=WGS84 +units=m +no_defs +type=crs"),
        32613 => Some("+proj=utm +zone=13 +datum=WGS84 +units=m +no_defs +type=crs"),
        32614 => Some("+proj=utm +zone=14 +datum=WGS84 +units=m +no_defs +type=crs"),
        32615 => Some("+proj=utm +zone=15 +datum=WGS84 +units=m +no_defs +type=crs"),
        32616 => Some("+proj=utm +zone=16 +datum=WGS84 +units=m +no_defs +type=crs"),
        32617 => Some("+proj=utm +zone=17 +datum=WGS84 +units=m +no_defs +type=crs"),
        32618 => Some("+proj=utm +zone=18 +datum=WGS84 +units=m +no_defs +type=crs"),
        32619 => Some("+proj=utm +zone=19 +datum=WGS84 +units=m +no_defs +type=crs"),
        32620 => Some("+proj=utm +zone=20 +datum=WGS84 +units=m +no_defs +type=crs"),

        // UTM Zones South (WGS 84) — 32701-32760 (subset)
        32701 => Some("+proj=utm +zone=1 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32702 => Some("+proj=utm +zone=2 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32703 => Some("+proj=utm +zone=3 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32704 => Some("+proj=utm +zone=4 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32705 => Some("+proj=utm +zone=5 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32706 => Some("+proj=utm +zone=6 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32707 => Some("+proj=utm +zone=7 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32708 => Some("+proj=utm +zone=8 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32709 => Some("+proj=utm +zone=9 +south +datum=WGS84 +units=m +no_defs +type=crs"),
        32710 => Some("+proj=utm +zone=10 +south +datum=WGS84 +units=m +no_defs +type=crs"),

        _ => None,
    }
}

/// Returns true if the given SRID represents a geographic (lon/lat in degrees) CRS.
pub fn is_geographic(srid: i32) -> bool {
    matches!(srid, 4326 | 4269 | 4267 | 4258 | 4148 | 4674 | 4283 | 4612 | 4490)
}

/// Returns true if the given SRID is in the known registry.
pub fn is_known_srid(srid: i32) -> bool {
    get_proj4_string(srid).is_some()
}

/// Returns a sorted list of all known SRID codes in the registry.
pub fn list_known_srids() -> Vec<i32> {
    let mut srids = vec![
        // Geographic
        4326, 4269, 4267, 4258, 4148, 4674, 4283, 4612, 4490,
        // Projected (global/continental)
        3857, 3395, 3035, 2154, 27700, 2163, 3408, 3409, 3410,
        // UTM North
        32601, 32602, 32603, 32604, 32605, 32606, 32607, 32608, 32609, 32610,
        32611, 32612, 32613, 32614, 32615, 32616, 32617, 32618, 32619, 32620,
        // UTM South
        32701, 32702, 32703, 32704, 32705, 32706, 32707, 32708, 32709, 32710,
    ];
    srids.sort();
    srids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_wgs84() {
        let proj4 = get_proj4_string(4326).unwrap();
        assert!(proj4.contains("+proj=longlat"));
        assert!(proj4.contains("WGS84"));
    }

    #[test]
    fn lookup_web_mercator() {
        let proj4 = get_proj4_string(3857).unwrap();
        assert!(proj4.contains("+proj=merc"));
    }

    #[test]
    fn lookup_nad83() {
        let proj4 = get_proj4_string(4269).unwrap();
        assert!(proj4.contains("+proj=longlat"));
        assert!(proj4.contains("NAD83"));
    }

    #[test]
    fn lookup_nad27() {
        let proj4 = get_proj4_string(4267).unwrap();
        assert!(proj4.contains("+proj=longlat"));
        assert!(proj4.contains("NAD27"));
    }

    #[test]
    fn lookup_utm_zone_18n() {
        let proj4 = get_proj4_string(32618).unwrap();
        assert!(proj4.contains("+proj=utm"));
        assert!(proj4.contains("+zone=18"));
        assert!(!proj4.contains("+south"));
    }

    #[test]
    fn lookup_utm_zone_10s() {
        let proj4 = get_proj4_string(32710).unwrap();
        assert!(proj4.contains("+proj=utm"));
        assert!(proj4.contains("+zone=10"));
        assert!(proj4.contains("+south"));
    }

    #[test]
    fn lookup_lambert_93() {
        let proj4 = get_proj4_string(2154).unwrap();
        assert!(proj4.contains("+proj=lcc"));
    }

    #[test]
    fn lookup_british_national_grid() {
        let proj4 = get_proj4_string(27700).unwrap();
        assert!(proj4.contains("+proj=tmerc"));
    }

    #[test]
    fn lookup_laea_europe() {
        let proj4 = get_proj4_string(3035).unwrap();
        assert!(proj4.contains("+proj=laea"));
    }

    #[test]
    fn lookup_us_national_atlas() {
        let proj4 = get_proj4_string(2163).unwrap();
        assert!(proj4.contains("+proj=laea"));
    }

    #[test]
    fn lookup_world_mercator() {
        let proj4 = get_proj4_string(3395).unwrap();
        assert!(proj4.contains("+proj=merc"));
    }

    #[test]
    fn lookup_unknown_srid_returns_none() {
        assert!(get_proj4_string(99999).is_none());
        assert!(get_proj4_string(0).is_none());
        assert!(get_proj4_string(-1).is_none());
    }

    #[test]
    fn all_known_srids_have_proj4_strings() {
        for srid in list_known_srids() {
            assert!(
                get_proj4_string(srid).is_some(),
                "SRID {} should have a proj4 string",
                srid
            );
        }
    }

    #[test]
    fn is_geographic_true_for_geographic_crs() {
        assert!(is_geographic(4326));
        assert!(is_geographic(4269));
        assert!(is_geographic(4267));
        assert!(is_geographic(4258));
        assert!(is_geographic(4148));
        assert!(is_geographic(4674));
        assert!(is_geographic(4283));
        assert!(is_geographic(4612));
        assert!(is_geographic(4490));
    }

    #[test]
    fn is_geographic_false_for_projected_crs() {
        assert!(!is_geographic(3857));
        assert!(!is_geographic(3395));
        assert!(!is_geographic(3035));
        assert!(!is_geographic(2154));
        assert!(!is_geographic(27700));
        assert!(!is_geographic(32618));
        assert!(!is_geographic(32710));
    }

    #[test]
    fn is_known_srid_true_for_registry_entries() {
        assert!(is_known_srid(4326));
        assert!(is_known_srid(3857));
        assert!(is_known_srid(32618));
    }

    #[test]
    fn is_known_srid_false_for_unknown() {
        assert!(!is_known_srid(99999));
        assert!(!is_known_srid(0));
    }

    #[test]
    fn list_known_srids_is_sorted() {
        let srids = list_known_srids();
        for window in srids.windows(2) {
            assert!(window[0] < window[1], "SRIDs should be sorted");
        }
    }

    #[test]
    fn list_known_srids_contains_expected_count() {
        let srids = list_known_srids();
        // 9 geographic + 3 global projected + 4 national + 3 NSIDC + 20 UTM N + 10 UTM S = 49
        // But 4148 and 4674 share pattern; verify actual count matches registry
        assert_eq!(srids.len(), 48);
    }

    #[test]
    fn nsidc_ease_grid_srids() {
        assert!(is_known_srid(3408));
        assert!(is_known_srid(3409));
        assert!(is_known_srid(3410));
        assert!(!is_geographic(3408));
        assert!(!is_geographic(3409));
        assert!(!is_geographic(3410));
    }
}
