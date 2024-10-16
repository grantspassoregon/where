use address::{
    trace_init, Addresses, BusinessMatchRecords, Cli, CommonAddresses, GeoAddresses,
    GrantsPassAddresses, GrantsPassSpatialAddresses, JosephineCountyAddresses,
    JosephineCountyAddresses2024, JosephineCountySpatialAddresses2024, LexisNexis, MatchRecords,
    Point, Portable, SpatialAddress, SpatialAddresses,
};
use aid::prelude::*;
use clap::Parser;
use tracing::{error, info, trace, warn};

fn main() -> Clean<()> {
    let cli = Cli::parse();
    trace_init();

    match cli.command.as_str() {
        "filter" => {
            if let Some(filter) = cli.filter {
                info!("Filtering records.");
                if cli.business {
                    let match_records = BusinessMatchRecords::from_csv(cli.source.clone())?;
                    info!("Source records read: {} entries.", match_records.len());
                    let mut filtered = match_records.filter(&filter);
                    info!("Records remaining: {} entries.", filtered.len());
                    filtered.to_csv(cli.output)?;
                } else {
                    let match_records = MatchRecords::from_csv(cli.source.clone())?;
                    info!("Source records read: {} entries.", match_records.len());
                    let mut filtered = match_records.clone().filter(&filter);
                    info!("Records remaining: {} entries.", filtered.len());
                    filtered.to_csv(cli.output)?;
                }
            } else {
                warn!("Filter parameter (-f or --filter) must be set.");
            }
        }
        "drift" => {
            info!("Calculating spatial drift between datasets.");
            trace!("Reading source addresses.");
            let mut source_addresses = SpatialAddresses::default();
            if let Some(source_type) = &cli.source_type {
                match source_type.as_str() {
                    "grants_pass" => {
                        source_addresses = SpatialAddresses::from(
                            &GrantsPassSpatialAddresses::from_csv(&cli.source)?[..],
                        )
                    }
                    "josephine_county" => {
                        source_addresses = SpatialAddresses::from(
                            &JosephineCountySpatialAddresses2024::from_csv(&cli.source)?[..],
                        )
                    }
                    _ => error!("Invalid source data type."),
                }
            } else {
                error!("No source data type provided.");
            }

            trace!("Reading target addresses.");
            let mut target_addresses = SpatialAddresses::default();
            if let Some(target) = &cli.target {
                if let Some(target_type) = &cli.target_type {
                    match target_type.as_str() {
                        "grants_pass" => {
                            target_addresses = SpatialAddresses::from(
                                &GrantsPassSpatialAddresses::from_csv(target)?[..],
                            )
                        }
                        "josephine_county" => {
                            target_addresses = SpatialAddresses::from(
                                &JosephineCountySpatialAddresses2024::from_csv(target)?[..],
                            )
                        }
                        _ => error!("Invalid target data type."),
                    }
                } else {
                    error!("No target data type provided.");
                }
            } else {
                error!("No target data specified.");
            }

            let mut deltas =
                <SpatialAddress as Point>::deltas(&source_addresses, &target_addresses, 99.0);
            deltas.to_csv(cli.output.clone())?;
        }
        "lexisnexis" => {
            info!("Reading source records.");
            let mut source_addresses = CommonAddresses::default();
            if let Some(source_type) = &cli.source_type {
                match source_type.as_str() {
                    "grants_pass" => {
                        source_addresses = CommonAddresses::from(
                            &GrantsPassAddresses::from_csv(cli.source.clone())?[..],
                        )
                    }
                    "josephine_county" => {
                        source_addresses = CommonAddresses::from(
                            &JosephineCountyAddresses2024::from_csv(cli.source.clone())?[..],
                        )
                    }
                    _ => error!("Unrecognized file format."),
                }
            }

            info!("Source records read: {} entries.", source_addresses.len());

            trace!("Reading exclusion addresses.");
            let mut target_addresses = CommonAddresses::default();
            if let Some(target) = &cli.target {
                if let Some(target_type) = &cli.target_type {
                    match target_type.as_str() {
                        "grants_pass" => {
                            target_addresses =
                                CommonAddresses::from(&GrantsPassAddresses::from_csv(target)?[..])
                        }
                        "josephine_county" => {
                            target_addresses = CommonAddresses::from(
                                &JosephineCountyAddresses::from_csv(target)?[..],
                            )
                        }
                        _ => error!("Invalid target data type."),
                    }
                } else {
                    error!("No target data type provided.");
                }
            } else {
                error!("No target data specified.");
            }
            info!(
                "Exclusion records read: {} entries.",
                target_addresses.len()
            );
            let mut lx = LexisNexis::from_addresses(&source_addresses, &target_addresses)?;
            lx.to_csv(cli.output)?;
        }
        "save" => {
            info!("Loading and saving addresses...");
            trace!("Reading source addresses.");
            let mut source_addresses = SpatialAddresses::default();
            if let Some(source_type) = &cli.source_type {
                match source_type.as_str() {
                    "grants_pass" => {
                        source_addresses = SpatialAddresses::from(
                            &GrantsPassSpatialAddresses::from_csv(&cli.source)?[..],
                        )
                    }
                    "josephine_county" => {
                        source_addresses = SpatialAddresses::from(
                            &JosephineCountySpatialAddresses2024::from_csv(&cli.source)?[..],
                        );
                        source_addresses.standardize();
                    }
                    _ => error!("Invalid source data type."),
                }
            } else {
                error!("No source data type provided.");
            }
            if !source_addresses.is_empty() {
                source_addresses.save(&cli.output)?;
                info!("Addresses saved to {:?}", &cli.output);
            } else {
                warn!("All records dropped.  Aborting save.");
            }
        }
        "orphan_streets" => {
            info!("Reading source records.");
            let mut source_addresses = CommonAddresses::default();
            if let Some(source_type) = &cli.source_type {
                match source_type.as_str() {
                    "grants_pass" => {
                        source_addresses = CommonAddresses::from(
                            &GrantsPassSpatialAddresses::from_csv(cli.source.clone())?[..],
                        )
                    }
                    "josephine_county" => {
                        source_addresses = CommonAddresses::from(
                            &JosephineCountySpatialAddresses2024::from_csv(cli.source.clone())?[..],
                        )
                    }
                    _ => error!("Unrecognized file format."),
                }
            }

            info!("Source records read: {} entries.", source_addresses.len());

            trace!("Reading exclusion addresses.");
            let mut target_addresses = CommonAddresses::default();
            if let Some(target) = &cli.target {
                if let Some(target_type) = &cli.target_type {
                    match target_type.as_str() {
                        "josephine_county" => {
                            target_addresses = CommonAddresses::from(
                                &JosephineCountySpatialAddresses2024::from_csv(target)?[..],
                            )
                        }
                        _ => error!("Invalid target data type."),
                    }
                } else {
                    error!("No target data type provided.");
                }
            } else {
                error!("No target data specified.");
            }
            info!(
                "Exclusion records read: {} entries.",
                target_addresses.len()
            );
            let orphans = &source_addresses.orphan_streets(&target_addresses);
            info!("{:?}", orphans);
        }
        "duplicates" => {
            info!("Reading source records.");
            let mut source_addresses = CommonAddresses::default();
            if let Some(source_type) = &cli.source_type {
                match source_type.as_str() {
                    "grants_pass" => {
                        source_addresses = CommonAddresses::from(
                            &GrantsPassSpatialAddresses::from_csv(cli.source.clone())?[..],
                        )
                    }
                    "josephine_county" => {
                        source_addresses = CommonAddresses::from(
                            &JosephineCountySpatialAddresses2024::from_csv(cli.source.clone())?[..],
                        )
                    }
                    _ => error!("Unrecognized file format."),
                }
            }

            info!("Source records read: {} entries.", source_addresses.len());
            info!("Screening addresses for duplicate records.");
            let mut duplicates = CommonAddresses::from(&source_addresses.filter("duplicate")[..]);
            info!("Duplicate records: {:?}", duplicates.len());
            info!("Output file: {:?}", cli.output);
            duplicates.to_csv(cli.output)?;
        }
        // "compare" => {
        //     if cli.business {
        //         info!("Matching business addresses.");
        //         info!("Reading source records.");
        //         let source_addresses = BusinessLicenses::from_csv(cli.source.clone())?;
        //         info!(
        //             "Source records read: {} entries.",
        //             source_addresses.records.len()
        //         );
        //         let source_addresses = source_addresses.deduplicate();
        //         info!(
        //             "Records deduplicated: {} remaining.",
        //             source_addresses.records.len()
        //         );
        //         info!("Reading comparison records.");
        //         let mut target_addresses = Addresses::default();
        //         if let Some(target) = &cli.target {
        //             if let Some(target_type) = &cli.target_type {
        //                 match target_type.as_str() {
        //                     "grants_pass" => {
        //                         target_addresses = Addresses::from(CityAddresses::from_csv(target)?)
        //                     }
        //                     "grants_pass_2022" => {
        //                         target_addresses =
        //                             Addresses::from(GrantsPass2022Addresses::from_csv(target)?)
        //                     }
        //                     _ => info!("Unrecognized file format."),
        //                 }
        //             }
        //             info!(
        //                 "Target records read: {} entries.",
        //                 target_addresses.records_ref().len()
        //             );
        //         }
        //         if let Some(alternate) = cli.alternate {
        //             info!("Comparing multiple targets.");
        //             let mut alt_target = Addresses::default();
        //             if let Some(target_type) = &cli.alternate_type {
        //                 match target_type.as_str() {
        //                     "grants_pass" => {
        //                         alt_target = Addresses::from(CityAddresses::from_csv(alternate)?)
        //                     }
        //                     "grants_pass_2022" => {
        //                         alt_target =
        //                             Addresses::from(GrantsPass2022Addresses::from_csv(alternate)?)
        //                     }
        //                     _ => error!("Unrecognized file format."),
        //                 }
        //             }
        //             info!(
        //                 "Alternate target records read: {} entries.",
        //                 alt_target.records_ref().len()
        //             );
        //             info!("Comparing records.");
        //             let mut match_records = BusinessMatchRecords::compare_chain(
        //                 &source_addresses,
        //                 &[&target_addresses, &alt_target],
        //             );
        //             info!("{:?} records categorized.", match_records.records.len());
        //             info!("Output file: {:?}", cli.output);
        //             match_records.to_csv(cli.output)?;
        //         } else {
        //             info!("Comparing records.");
        //             let mut match_records =
        //                 BusinessMatchRecords::compare(&source_addresses, &target_addresses);
        //             info!("{:?} records categorized.", match_records.records.len());
        //             info!("Output file: {:?}", cli.output);
        //             match_records.to_csv(cli.output)?;
        //         }
        //     } else {
        //         info!("Matching addresses.");
        //         info!("Reading source records.");
        //         let mut source_addresses = Addresses::default();
        //         if let Some(source_type) = &cli.source_type {
        //             match source_type.as_str() {
        //                 "grants_pass" => {
        //                     source_addresses = Addresses::from(CityAddresses::from_csv(cli.source)?)
        //                 }
        //                 "grants_pass_2022" => {
        //                     source_addresses =
        //                         Addresses::from(GrantsPass2022Addresses::from_csv(cli.source)?)
        //                 }
        //                 "josephine_county" => {
        //                     source_addresses =
        //                         Addresses::from(CountyAddresses::from_csv(cli.source)?)
        //                 }
        //                 _ => error!("Unrecognized file format."),
        //             }
        //         }
        //
        //         info!(
        //             "Source records read: {} entries.",
        //             source_addresses.records_ref().len()
        //         );
        //         if cli.duplicates {
        //             info!("Screening for duplicate records.");
        //             let mut same = source_addresses.filter("duplicate");
        //             info!("Duplicate records: {:?}", same.records_ref().len());
        //             info!("Output file: {:?}", cli.output);
        //             same.to_csv(cli.output)?;
        //         } else if let Some(target) = cli.target {
        //             info!("Reading comparison records.");
        //             let mut target_addresses = Addresses::default();
        //             if let Some(target_type) = cli.target_type {
        //                 match target_type.as_str() {
        //                     "grants_pass" => {
        //                         target_addresses = Addresses::from(CityAddresses::from_csv(target)?)
        //                     }
        //                     "grants_pass_2022" => {
        //                         target_addresses =
        //                             Addresses::from(GrantsPass2022Addresses::from_csv(target)?)
        //                     }
        //                     "josephine_county" => {
        //                         target_addresses =
        //                             Addresses::from(CountyAddresses::from_csv(target)?)
        //                     }
        //                     _ => error!("Unrecognized file format."),
        //                 }
        //             }
        //             info!(
        //                 "Comparison records read: {} entries.",
        //                 target_addresses.records_ref().len()
        //             );
        //             info!("Comparing records.");
        //             let mut match_records = MatchRecords::compare(
        //                 source_addresses.records_ref(),
        //                 target_addresses.records_ref(),
        //             );
        //             info!(
        //                 "{:?} records categorized.",
        //                 match_records.records_ref().len()
        //             );
        //             info!("Output file: {:?}", cli.output);
        //             match_records.to_csv(cli.output)?;
        //         }
        //     }
        // }
        "compare" => {
            info!("Reading source records.");
            let mut source = GeoAddresses::default();
            if let Some(source_type) = &cli.source_type {
                match source_type.as_str() {
                    "grants_pass" => {
                        source = GeoAddresses::from(
                            &GrantsPassSpatialAddresses::from_csv(cli.source.clone())?[..],
                        )
                    }
                    "josephine_county" => {
                        source = GeoAddresses::from(
                            &JosephineCountySpatialAddresses2024::from_csv(cli.source.clone())?[..],
                        )
                    }
                    _ => error!("Unrecognized file format."),
                }
            }
            info!("Reading target records.");
            let mut target = GeoAddresses::default();
            if let Some(target_type) = &cli.target_type {
                if let Some(target_path) = &cli.target {
                    match target_type.as_str() {
                        "grants_pass" => {
                            target = GeoAddresses::from(
                                &GrantsPassSpatialAddresses::from_csv(target_path)?[..],
                            )
                        }
                        "josephine_county" => {
                            target = GeoAddresses::from(
                                &JosephineCountySpatialAddresses2024::from_csv(target_path)?[..],
                            );
                            target.standardize();
                        }
                        _ => error!("Unrecognized file format."),
                    }
                }
            }
            info!("Comparing records.");

            info!("Remove retired addresses from source.");
            info!("Source records prior: {}", source.len());
            source.filter_field("active", "");
            // source = GeoAddresses::from(&source.filter("active")[..]);
            info!("Source records post: {}", source.len());

            let mut match_records = MatchRecords::compare(&source, &target);
            info!("{:?} records categorized.", match_records.len());
            info!("Output file: {:?}", cli.output);
            match_records.to_csv(cli.output)?;
        }
        _ => {}
    }

    Ok(())
}
