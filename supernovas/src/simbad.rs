//! Utilities for querying the SIMBAD catalog entries

use std::io::BufReader;

use crate::positions::CatalogEntry;
use quick_xml::{events::Event, reader::Reader};

impl CatalogEntry {
    /// Construct a [`CatalogEntry`] from a SIMBAD query
    pub fn from_simbad(ident: &str, catalog: &str) -> super::Result<Self> {
        // By default, this is in ICRS, J2000
        let query_string = format!(
            "https://simbad.cds.unistra.fr/simbad/sim-id?output.format=votable&Ident={ident}&output.params=main_id,id({catalog}),ra,dec,pmra,pmdec,plx,rv_value"
        );
        let resp = reqwest::blocking::get(query_string)?;
        let bufread = BufReader::new(resp);
        let mut xml_reader = Reader::from_reader(bufread);

        let mut columns = Vec::new();
        let mut buf = Vec::new();

        let mut td_read = false;
        let mut td_text = false;
        // Seek to the table and pull out all the table entries
        loop {
            match xml_reader.read_event_into(&mut buf) {
                // exits the loop when reaching end of file
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    if matches!(e.name().as_ref(), b"TD") {
                        td_read = true;
                    }
                }
                Ok(Event::Text(e)) => {
                    if td_read {
                        columns.push(e.unescape().unwrap().into_owned());
                        td_text = true;
                    }
                }
                Ok(Event::End(e)) => {
                    if matches!(e.name().as_ref(), b"TD") {
                        td_read = false;
                        if !td_text {
                            // Empty columns
                            columns.push("".to_string());
                        }
                        td_text = false;
                    }
                }
                _ => (),
            }
            buf.clear();
        }

        // Parse catalog info
        let (cat, num) = if let Some((cat, id)) = columns[1].split_once(' ') {
            (cat, id.parse().expect("Invalid catalog ID"))
        } else {
            ("", 0)
        };

        // Parse RA and DEC
        let ra_parts = columns[2]
            .split_whitespace()
            .into_iter()
            .collect::<Vec<_>>();
        let (ra_h, ra_m, ra_s) = (
            ra_parts[0].parse().expect("invalid ra_h"),
            ra_parts[1].parse().expect("invalid ra_m"),
            ra_parts[2].parse().expect("invalid ra_s"),
        );

        let dec_parts = columns[3]
            .split_whitespace()
            .into_iter()
            .collect::<Vec<_>>();
        let (dec_d, dec_m, dec_s) = (
            dec_parts[0].parse().expect("invalid dec_d"),
            dec_parts[1].parse().expect("invalid dec_m"),
            dec_parts[2].parse().expect("invalid dec_s"),
        );

        // SIMBAD appends NAME to qualify common or historical names, which we want to drop
        let name = if columns[0].starts_with("NAME") {
            columns[0].strip_prefix("NAME ").unwrap().to_string()
        } else {
            columns[0].to_string()
        };

        CatalogEntry::new_hms(
            &name,
            cat,
            num,
            (ra_h, ra_m, ra_s),
            (dec_d, dec_m, dec_s),
            columns[4].parse().unwrap_or(0.0),
            columns[5].parse().unwrap_or(0.0),
            columns[6].parse().unwrap_or(0.0),
            columns[7].parse().unwrap_or(0.0),
        )
    }
}
