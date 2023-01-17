use crate::address_components::*;
use crate::utils::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Address {
    address_number: i64,
    address_number_suffix: Option<String>,
    street_name_pre_directional: Option<StreetNamePreDirectional>,
    street_name: String,
    street_name_post_type: StreetNamePostType,
    subaddress_type: Option<SubaddressType>,
    subaddress_identifier: Option<String>,
    floor: Option<i64>,
    building: Option<String>,
    zip_code: i64,
    postal_community: String,
    state_name: String,
    status: AddressStatus,
    object_id: i64,
}

impl Address {
    pub fn coincident(self, other: &Address) -> AddressMatch {
        let mut coincident = false;
        let mut mismatches = Vec::new();
        if self.address_number == other.address_number
            && self.address_number_suffix == other.address_number_suffix
            && self.street_name_pre_directional == other.street_name_pre_directional
            && self.street_name == other.street_name
            && self.street_name_post_type == other.street_name_post_type
            && self.subaddress_identifier == other.subaddress_identifier
            && self.zip_code == other.zip_code
            && self.postal_community == other.postal_community
            && self.state_name == other.state_name
        {
            coincident = true;
            if self.subaddress_type != other.subaddress_type {
                mismatches.push(Mismatch::subaddress_type(
                    self.subaddress_type,
                    other.subaddress_type,
                ));
            }
            if self.floor != other.floor {
                mismatches.push(Mismatch::floor(self.floor, other.floor));
            }
            if self.building != other.building {
                mismatches.push(Mismatch::building(self.building, other.building.clone()));
            }
            if self.status != other.status {
                mismatches.push(Mismatch::status(self.status, other.status));
            }
        }
        AddressMatch::new(coincident, mismatches)
    }

    fn label(&self) -> String {
        let complete_address_number = match &self.address_number_suffix {
            Some(suffix) => format!("{} {}", self.address_number, suffix),
            None => self.address_number.to_string(),
        };
        let complete_street_name = match self.street_name_pre_directional {
            Some(pre_directional) => format!(
                "{:?} {} {:?}",
                pre_directional, self.street_name, self.street_name_post_type
            ),
            None => format!("{} {:?}", self.street_name, self.street_name_post_type),
        };
        let complete_subaddress = match &self.subaddress_identifier {
            Some(identifier) => match self.subaddress_type {
                Some(subaddress_type) => Some(format!("{:?} {}", subaddress_type, identifier)),
                None => Some(format!("#{}", identifier)),
            },
            None => self
                .subaddress_type
                .map(|subaddress_type| format!("{:?}", subaddress_type)),
        };
        match complete_subaddress {
            Some(subaddress) => format!(
                "{} {} {}",
                complete_address_number, complete_street_name, subaddress
            ),
            None => format!("{} {}", complete_address_number, complete_street_name),
        }
    }
}

impl From<CityAddress> for Address {
    fn from(item: CityAddress) -> Self {
        Address {
            address_number: item.address_number,
            address_number_suffix: item.address_number_suffix,
            street_name_pre_directional: item.street_name_pre_directional,
            street_name: item.street_name,
            street_name_post_type: item.street_name_post_type,
            subaddress_type: item.subaddress_type,
            subaddress_identifier: item.subaddress_identifier,
            floor: item.floor,
            building: item.building,
            zip_code: item.zip_code,
            postal_community: item.postal_community,
            state_name: item.state_name,
            status: item.status,
            object_id: item.object_id,
        }
    }
}

impl TryFrom<CountyAddress> for Address {
    type Error = ();

    fn try_from(item: CountyAddress) -> Result<Self, Self::Error> {
        match item.street_name_post_type {
            Some(post_type) => Ok(Address {
                address_number: item.address_number,
                address_number_suffix: item.address_number_suffix,
                street_name_pre_directional: item.street_name_pre_directional,
                street_name: item.street_name,
                street_name_post_type: post_type,
                subaddress_type: item.subaddress_type,
                subaddress_identifier: item.subaddress_identifier,
                floor: item.floor,
                building: None,
                zip_code: item.zip_code,
                postal_community: item.postal_community,
                state_name: item.state_name,
                status: item.status,
                object_id: item.object_id,
            }),
            None => Err(()),
        }
    }
}

pub enum Mismatch {
    SubaddressType(String),
    Floor(String),
    Building(String),
    Status(String),
}

impl Mismatch {
    fn subaddress_type(from: Option<SubaddressType>, to: Option<SubaddressType>) -> Self {
        let message = format!("{:?} not equal to {:?}", from, to);
        Self::SubaddressType(message)
    }

    fn floor(from: Option<i64>, to: Option<i64>) -> Self {
        let message = format!("{:?} not equal to {:?}", from, to);
        Self::Floor(message)
    }

    fn building(from: Option<String>, to: Option<String>) -> Self {
        let message = format!("{:?} not equal to {:?}", from, to);
        Self::Building(message)
    }

    fn status(from: AddressStatus, to: AddressStatus) -> Self {
        let message = format!("{:?} not equal to {:?}", from, to);
        Self::Status(message)
    }
}

struct Mismatches {
    fields: Vec<Mismatch>,
}

impl Mismatches {
    fn new(fields: Vec<Mismatch>) -> Self {
        Mismatches { fields }
    }
}

pub struct AddressMatch {
    coincident: bool,
    mismatches: Option<Mismatches>,
}

impl AddressMatch {
    fn new(coincident: bool, fields: Vec<Mismatch>) -> Self {
        let mismatches = match fields.len() {
            0 => None,
            _ => Some(Mismatches::new(fields)),
        };
        AddressMatch {
            coincident,
            mismatches,
        }
    }
}

#[derive(Debug)]
pub enum MatchStatus {
    Matching,
    Divergent,
    Missing,
}

#[derive(Debug)]
pub struct MatchRecord {
    pub match_status: MatchStatus,
    pub address_label: String,
    pub self_id: i64,
    pub other_id: Option<i64>,
    pub subaddress_type: Option<String>,
    pub floor: Option<String>,
    pub building: Option<String>,
    pub status: Option<String>,
}

pub struct MatchRecords {
    pub records: Vec<MatchRecord>,
}

impl MatchRecords {
    pub fn new<A: Into<Address> + Clone, B: TryInto<Address> + Clone>(
        self_address: A,
        other_addresses: Vec<B>,
    ) -> Self {
        let self_address = self_address.into();
        let self_id = self_address.object_id;
        let address_label = self_address.label();

        let mut match_record = Vec::new();

        for address in other_addresses {
            if let Ok(other) = address.clone().try_into() {
                let address_match = self_address.clone().coincident(&other);
                if address_match.coincident {
                    let other_id = Some(other.clone().object_id);
                    let mut subaddress_type = None;
                    let mut floor = None;
                    let mut building = None;
                    let mut status = None;
                    match address_match.mismatches {
                        None => match_record.push(MatchRecord {
                            match_status: MatchStatus::Matching,
                            address_label: address_label.clone(),
                            self_id,
                            other_id,
                            subaddress_type,
                            floor,
                            building,
                            status,
                        }),
                        Some(mismatches) => {
                            for mismatch in mismatches.fields {
                                match mismatch {
                                    Mismatch::SubaddressType(message) => {
                                        subaddress_type = Some(message)
                                    }
                                    Mismatch::Floor(message) => floor = Some(message),
                                    Mismatch::Building(message) => building = Some(message),
                                    Mismatch::Status(message) => status = Some(message),
                                }
                            }
                            match_record.push(MatchRecord {
                                match_status: MatchStatus::Divergent,
                                address_label: address_label.clone(),
                                self_id,
                                other_id,
                                subaddress_type,
                                floor,
                                building,
                                status,
                            })
                        }
                    }
                }
            }
        }
        if match_record.is_empty() {
            match_record.push(MatchRecord {
                match_status: MatchStatus::Missing,
                address_label,
                self_id,
                other_id: None,
                subaddress_type: None,
                floor: None,
                building: None,
                status: None,
            })
        }
        MatchRecords {
            records: match_record,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CityAddress {
    #[serde(rename(deserialize = "OID_"))]
    object_id: i64,
    #[serde(rename(deserialize = "Add_Number"))]
    address_number: i64,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "AddNum_Suf")
    )]
    address_number_suffix: Option<String>,
    #[serde(
        deserialize_with = "csv::invalid_option",
        rename(deserialize = "St_PreDir")
    )]
    street_name_pre_directional: Option<StreetNamePreDirectional>,
    #[serde(rename(deserialize = "St_Name"))]
    street_name: String,
    #[serde(rename(deserialize = "St_PosTyp"))]
    street_name_post_type: StreetNamePostType,
    #[serde(deserialize_with = "csv::invalid_option")]
    subaddress_type: Option<SubaddressType>,
    #[serde(deserialize_with = "deserialize_arcgis_data")]
    subaddress_identifier: Option<String>,
    #[serde(deserialize_with = "csv::invalid_option")]
    floor: Option<i64>,
    #[serde(deserialize_with = "deserialize_arcgis_data")]
    building: Option<String>,
    #[serde(rename(deserialize = "Post_Code"))]
    zip_code: i64,
    #[serde(rename(deserialize = "STATUS"))]
    status: AddressStatus,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "NOTIFICATION")
    )]
    notification: Option<String>,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "NOTES")
    )]
    notes: Option<String>,
    #[serde(rename(deserialize = "GlobalID"))]
    global_id: String,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "created_user")
    )]
    created_user: Option<String>,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "created_date")
    )]
    created_date: Option<String>,
    #[serde(rename(deserialize = "last_edited_user"))]
    last_edited_user: String,
    #[serde(rename(deserialize = "last_edited_date"))]
    last_edited_date: String,
    complete_address_number: String,
    complete_street_name: String,
    #[serde(deserialize_with = "deserialize_arcgis_data")]
    complete_subaddress: Option<String>,
    complete_street_address: String,
    street_address_label: String,
    place_state_zip: String,
    #[serde(rename(deserialize = "Post_Comm"))]
    postal_community: String,
    state_name: String,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "Inc_Muni")
    )]
    incorporated_municipality: Option<String>,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "Uninc_Comm")
    )]
    unincorporated_community: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CityAddresses {
    pub records: Vec<CityAddress>,
}

impl CityAddresses {
    pub fn from_csv<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let mut data = Vec::new();
        let file = std::fs::File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: CityAddress = result?;
            data.push(record);
        }

        Ok(CityAddresses { records: data })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CountyAddress {
    #[serde(rename(deserialize = "OID_"))]
    object_id: i64,
    #[serde(deserialize_with = "deserialize_arcgis_data")]
    taxlot: Option<String>,
    #[serde(rename(deserialize = "stnum"))]
    address_number: i64,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "stnumsuf")
    )]
    address_number_suffix: Option<String>,
    #[serde(
        deserialize_with = "deserialize_abbreviated_pre_directional",
        rename(deserialize = "predir")
    )]
    street_name_pre_directional: Option<StreetNamePreDirectional>,
    #[serde(rename(deserialize = "name"))]
    street_name: String,
    #[serde(
        deserialize_with = "deserialize_abbreviated_post_type",
        rename(deserialize = "type")
    )]
    street_name_post_type: Option<StreetNamePostType>,
    #[serde(
        deserialize_with = "deserialize_abbreviated_subaddress_type",
        rename(deserialize = "unit_type")
    )]
    subaddress_type: Option<SubaddressType>,
    #[serde(
        deserialize_with = "deserialize_arcgis_data",
        rename(deserialize = "unit")
    )]
    subaddress_identifier: Option<String>,
    #[serde(deserialize_with = "zero_floor")]
    floor: Option<i64>,
    #[serde(rename(deserialize = "address"))]
    complete_street_address: String,
    #[serde(rename(deserialize = "postcomm"))]
    postal_community: String,
    #[serde(rename(deserialize = "zip"))]
    zip_code: i64,
    #[serde(rename(deserialize = "state"))]
    state_name: String,
    status: AddressStatus,
    point_y: Option<f64>,
    point_x: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CountyAddresses {
    pub records: Vec<CountyAddress>,
}

impl CountyAddresses {
    pub fn from_csv<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let mut data = Vec::new();
        let file = std::fs::File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: CountyAddress = result?;
            data.push(record);
        }

        Ok(CountyAddresses { records: data })
    }
}
