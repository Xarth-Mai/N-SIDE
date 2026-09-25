use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
};

use bevy::prelude::Vec3;
use geo::{Area, LineString, Polygon, Validation};
use serde::{Deserialize, Deserializer, de};
use serde_json::Value;

/// Authoritative spatial data; editorial metadata and interiors stay in the source document
#[derive(Clone, Debug, Deserialize)]
pub struct Map {
    pub version: u32,
    pub units: Units,
    pub terrain: Terrain,
    #[serde(deserialize_with = "unique_nodes")]
    pub nodes: BTreeMap<String, [f64; 3]>,
    pub roads: Vec<Road>,
    pub buildings: Vec<Building>,
    pub surfaces: Vec<Surface>,
    pub trees: Vec<[f64; 2]>,
    #[serde(rename = "elevatedNodes")]
    pub elevated_nodes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Units {
    pub horizontal: String,
    pub vertical: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Terrain {
    pub water: Vec<[f64; 2]>,
    pub samples: Vec<[f64; 3]>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Road {
    pub nodes: Vec<String>,
    pub kind: String,
    pub width: f64,
    pub building: Option<String>,
    pub surface: Option<String>,
    pub access: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Building {
    pub id: String,
    pub polygon: Vec<[f64; 2]>,
    pub elevation: f64,
    pub height: f64,
    pub kind: String,
    pub bank: String,
    pub design: Option<Design>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Design {
    pub floors: Vec<Floor>,
    pub entries: Vec<Entry>,
    pub front: Option<[[f64; 2]; 2]>,
    pub canopy: Option<f64>,
    pub lightwell: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Floor {
    pub name: String,
    pub z: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    pub node: String,
    pub role: String,
    pub level: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Surface {
    pub id: Option<String>,
    pub place: Option<String>,
    pub kind: String,
    pub elevation: f64,
    pub polygon: Vec<[f64; 2]>,
    pub building: Option<String>,
    #[serde(default)]
    pub elevated: bool,
    #[serde(rename = "baseElevation")]
    pub base_elevation: Option<f64>,
    pub boundary: Option<Vec<[f64; 2]>>,
    pub access: Option<String>,
}

/// The only map/engine axis conversion: source [east, north, height] in meters
pub fn map_to_world([x, y, h]: [f64; 3]) -> Vec3 {
    Vec3::new(x as f32, h as f32, -y as f32)
}

pub fn world_to_map(position: Vec3) -> [f64; 3] {
    [position.x as f64, -position.z as f64, position.y as f64]
}

#[derive(Debug)]
pub struct MapError {
    pub file: PathBuf,
    pub pointer: String,
    pub object_id: Option<String>,
    pub actual: String,
    pub cause: String,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.file.display(), self.pointer)?;
        if let Some(id) = &self.object_id {
            write!(f, " (object {id})")?;
        }
        write!(f, ": {}; actual={}", self.cause, self.actual)
    }
}

impl std::error::Error for MapError {}

impl Map {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, MapError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|error| MapError {
            file: path.to_owned(),
            pointer: String::new(),
            object_id: None,
            actual: "<unread>".into(),
            cause: format!("cannot read map: {error}"),
        })?;
        Self::parse(&text, path)
    }

    pub fn parse(text: &str, path: impl AsRef<Path>) -> Result<Self, MapError> {
        let context = Context {
            file: path.as_ref(),
            text,
        };
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let map: Self = serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
            let pointer = error
                .path()
                .iter()
                .map(|segment| match segment {
                    serde_path_to_error::Segment::Seq { index } => format!("/{index}"),
                    serde_path_to_error::Segment::Map { key }
                    | serde_path_to_error::Segment::Enum { variant: key } => {
                        format!("/{}", escape(key))
                    }
                    serde_path_to_error::Segment::Unknown => "/?".into(),
                })
                .collect::<String>();
            context.error(&pointer, format!("cannot parse map: {}", error.inner()))
        })?;
        deserializer
            .end()
            .map_err(|error| context.error("", error.to_string()))?;
        map.validate(&context)?;
        Ok(map)
    }

    fn validate(&self, c: &Context<'_>) -> Result<(), MapError> {
        if self.version != 7 {
            return Err(c.error("/version", "unsupported map version; expected 7"));
        }
        c.choice("/units/horizontal", &self.units.horizontal, &["m"])?;
        c.choice("/units/vertical", &self.units.vertical, &["m"])?;
        c.polygon("/terrain/water", &self.terrain.water)?;
        if self.terrain.samples.len() < 3 {
            return Err(c.error(
                "/terrain/samples",
                "at least three terrain controls required",
            ));
        }
        for (i, sample) in self.terrain.samples.iter().enumerate() {
            c.point(&format!("/terrain/samples/{i}"), sample)?;
        }
        if self.nodes.is_empty() {
            return Err(c.error("/nodes", "at least one map node required"));
        }
        for (name, point) in &self.nodes {
            let p = format!("/nodes/{}", escape(name));
            if name.is_empty() {
                return Err(c.error(&p, "node ID must not be empty"));
            }
            c.point(&p, point)?;
        }
        let mut building_ids = BTreeSet::new();
        for (i, building) in self.buildings.iter().enumerate() {
            let p = format!("/buildings/{i}");
            c.unique_id(&format!("{p}/id"), &building.id, &mut building_ids)?;
            c.choice(
                &format!("{p}/kind"),
                &building.kind,
                &[
                    "apartment",
                    "civic",
                    "home",
                    "school",
                    "shop",
                    "shrine",
                    "station",
                ],
            )?;
            c.choice(
                &format!("{p}/bank"),
                &building.bank,
                &["district", "opposite"],
            )?;
            c.number(&format!("{p}/elevation"), building.elevation)?;
            c.positive(&format!("{p}/height"), building.height)?;
            c.number(&format!("{p}/height"), building.elevation + building.height)?;
            c.polygon(&format!("{p}/polygon"), &building.polygon)?;
            if let Some(design) = &building.design {
                let p = format!("{p}/design");
                if design.floors.is_empty() {
                    return Err(
                        c.error(&format!("{p}/floors"), "a building design requires floors")
                    );
                }
                let mut floor_names = BTreeSet::new();
                let mut previous_z = f64::NEG_INFINITY;
                for (j, floor) in design.floors.iter().enumerate() {
                    let q = format!("{p}/floors/{j}");
                    c.unique_id(&format!("{q}/name"), &floor.name, &mut floor_names)?;
                    c.number(&format!("{q}/z"), floor.z)?;
                    if floor.z < building.elevation
                        || floor.z > building.elevation + building.height
                        || floor.z <= previous_z
                    {
                        return Err(c.error(
                            &format!("{q}/z"),
                            "floors must ascend within the building elevation and roof",
                        ));
                    }
                    previous_z = floor.z;
                }
                for (j, entry) in design.entries.iter().enumerate() {
                    let q = format!("{p}/entries/{j}");
                    self.node_ref(c, &format!("{q}/node"), &entry.node)?;
                    c.choice(
                        &format!("{q}/role"),
                        &entry.role,
                        &["public", "resident", "service", "student"],
                    )?;
                    if !floor_names.contains(entry.level.as_str()) {
                        return Err(
                            c.error(&format!("{q}/level"), "entry references an unknown floor")
                        );
                    }
                }
                if let Some(front) = &design.front {
                    c.line(&format!("{p}/front"), front)?;
                }
                if let Some(canopy) = design.canopy {
                    c.number(&format!("{p}/canopy"), canopy)?;
                    if canopy < 0.0 {
                        return Err(
                            c.error(&format!("{p}/canopy"), "canopy depth must be nonnegative")
                        );
                    }
                }
                if let Some(hole) = &design.lightwell {
                    c.polygon(&format!("{p}/lightwell"), hole)?;
                    Polygon::new(ring(&building.polygon), vec![ring(hole)])
                        .check_validation()
                        .map_err(|error| c.error(&format!("{p}/lightwell"), error.to_string()))?;
                }
            }
        }
        let mut surface_ids = BTreeSet::new();
        for (i, surface) in self.surfaces.iter().enumerate() {
            let p = format!("/surfaces/{i}");
            if let Some(id) = &surface.id {
                c.unique_id(&format!("{p}/id"), id, &mut surface_ids)?;
            }
            c.choice(
                &format!("{p}/kind"),
                &surface.kind,
                &["court", "park", "platform", "private", "service"],
            )?;
            c.number(&format!("{p}/elevation"), surface.elevation)?;
            c.polygon(&format!("{p}/polygon"), &surface.polygon)?;
            c.reference(
                &format!("{p}/building"),
                surface.building.as_deref(),
                &building_ids,
            )?;
            c.access(&format!("{p}/access"), surface.access.as_deref())?;
            if let Some(base) = surface.base_elevation {
                c.number(&format!("{p}/baseElevation"), base)?;
                if base >= surface.elevation {
                    return Err(c.error(
                        &format!("{p}/baseElevation"),
                        "surface base must be below its elevation",
                    ));
                }
            }
            if let Some(boundary) = &surface.boundary {
                c.line(&format!("{p}/boundary"), boundary)?;
            }
        }
        for (i, road) in self.roads.iter().enumerate() {
            let p = format!("/roads/{i}");
            c.choice(
                &format!("{p}/kind"),
                &road.kind,
                &[
                    "avenue", "bridge", "crossing", "deck", "interior", "landing", "lane", "lift",
                    "main", "service", "shore", "steps", "trail",
                ],
            )?;
            c.positive(&format!("{p}/width"), road.width)?;
            if road.nodes.len() < 2 {
                return Err(c.error(&format!("{p}/nodes"), "road requires at least two nodes"));
            }
            for (j, node) in road.nodes.iter().enumerate() {
                self.node_ref(c, &format!("{p}/nodes/{j}"), node)?;
                if j > 0 {
                    let a = self.nodes[&road.nodes[j - 1]];
                    let b = self.nodes[node];
                    if a == b || (road.kind != "lift" && a[..2] == b[..2]) {
                        return Err(c.error(
                            &format!("{p}/nodes/{j}"),
                            "road segment has zero length (only lift permits vertical segments)",
                        ));
                    }
                }
            }
            c.reference(
                &format!("{p}/building"),
                road.building.as_deref(),
                &building_ids,
            )?;
            c.reference(
                &format!("{p}/surface"),
                road.surface.as_deref(),
                &surface_ids,
            )?;
            c.access(&format!("{p}/access"), road.access.as_deref())?;
        }
        let mut elevated = BTreeSet::new();
        for (i, name) in self.elevated_nodes.iter().enumerate() {
            let p = format!("/elevatedNodes/{i}");
            self.node_ref(c, &p, name)?;
            c.unique_id(&p, name, &mut elevated)?;
        }
        for (i, tree) in self.trees.iter().enumerate() {
            c.point(&format!("/trees/{i}"), tree)?;
        }
        Ok(())
    }

    fn node_ref(&self, c: &Context<'_>, path: &str, name: &str) -> Result<(), MapError> {
        if self.nodes.contains_key(name) {
            Ok(())
        } else {
            Err(c.error(path, "reference to missing node"))
        }
    }
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn ring(points: &[[f64; 2]]) -> LineString<f64> {
    LineString::from(points.iter().map(|p| (p[0], p[1])).collect::<Vec<_>>())
}

struct Context<'a> {
    file: &'a Path,
    text: &'a str,
}

impl Context<'_> {
    fn error(&self, pointer: &str, cause: impl Into<String>) -> MapError {
        let value = serde_json::from_str::<Value>(self.text).ok();
        let mut object_id = None;
        if let Some(value) = &value {
            let mut ancestor = pointer;
            loop {
                if let Some(id) = value
                    .pointer(ancestor)
                    .and_then(|v| v.get("id"))
                    .and_then(Value::as_str)
                {
                    object_id = Some(id.to_owned());
                    break;
                }
                if let Some((parent, _)) = ancestor.rsplit_once('/') {
                    ancestor = parent;
                } else {
                    break;
                }
            }
            if object_id.is_none() && pointer.starts_with("/nodes/") {
                object_id = pointer
                    .split('/')
                    .nth(2)
                    .map(|key| key.replace("~1", "/").replace("~0", "~"));
            }
        }
        let actual = value
            .as_ref()
            .and_then(|v| v.pointer(pointer))
            .map(|v| {
                let text = v.to_string();
                if text.chars().count() <= 200 {
                    text
                } else {
                    format!("{}…", text.chars().take(200).collect::<String>())
                }
            })
            .unwrap_or_else(|| "<unavailable>".into());
        MapError {
            file: self.file.to_owned(),
            pointer: pointer.into(),
            object_id,
            actual,
            cause: cause.into(),
        }
    }

    fn choice(&self, path: &str, value: &str, allowed: &[&str]) -> Result<(), MapError> {
        if allowed.contains(&value) {
            Ok(())
        } else {
            Err(self.error(
                path,
                format!("unsupported value; expected {}", allowed.join(", ")),
            ))
        }
    }

    fn access(&self, path: &str, access: Option<&str>) -> Result<(), MapError> {
        if let Some(access) = access {
            self.choice(
                path,
                access,
                &["public", "resident", "controlled", "service"],
            )?;
        }
        Ok(())
    }

    fn number(&self, path: &str, value: f64) -> Result<(), MapError> {
        if value.is_finite() && (value as f32).is_finite() {
            Ok(())
        } else {
            Err(self.error(
                path,
                "number must be finite and representable by the renderer",
            ))
        }
    }

    fn positive(&self, path: &str, value: f64) -> Result<(), MapError> {
        self.number(path, value)?;
        if value > 0.0 {
            Ok(())
        } else {
            Err(self.error(path, "value must be greater than zero"))
        }
    }

    fn point<const N: usize>(&self, path: &str, point: &[f64; N]) -> Result<(), MapError> {
        for (i, value) in point.iter().enumerate() {
            self.number(&format!("{path}/{i}"), *value)?;
        }
        Ok(())
    }

    fn line(&self, path: &str, points: &[[f64; 2]]) -> Result<(), MapError> {
        if points.len() < 2 {
            return Err(self.error(path, "line requires at least two points"));
        }
        for (i, point) in points.iter().enumerate() {
            self.point(&format!("{path}/{i}"), point)?;
            if i > 0 && points[i - 1] == *point {
                return Err(self.error(&format!("{path}/{i}"), "line segment has zero length"));
            }
        }
        Ok(())
    }

    fn polygon(&self, path: &str, points: &[[f64; 2]]) -> Result<(), MapError> {
        self.line(path, points)?;
        let polygon = Polygon::new(ring(points), vec![]);
        polygon
            .check_validation()
            .map_err(|error| self.error(path, error.to_string()))?;
        let area = polygon.unsigned_area();
        if !area.is_finite() || area <= 0.0 {
            return Err(self.error(path, "polygon area must be finite and greater than zero"));
        }
        Ok(())
    }

    fn unique_id<'a>(
        &self,
        path: &str,
        id: &'a str,
        ids: &mut BTreeSet<&'a str>,
    ) -> Result<(), MapError> {
        if id.is_empty() || !ids.insert(id) {
            Err(self.error(path, "ID must be nonempty and unique"))
        } else {
            Ok(())
        }
    }

    fn reference(
        &self,
        path: &str,
        id: Option<&str>,
        ids: &BTreeSet<&str>,
    ) -> Result<(), MapError> {
        if id.is_some_and(|id| !ids.contains(id)) {
            Err(self.error(path, "reference to missing object"))
        } else {
            Ok(())
        }
    }
}

fn unique_nodes<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, [f64; 3]>, D::Error> {
    struct Nodes;
    impl<'de> de::Visitor<'de> for Nodes {
        type Value = BTreeMap<String, [f64; 3]>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("an object of unique node IDs and three-dimensional coordinates")
        }
        fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut nodes = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, [f64; 3]>()? {
                if nodes.insert(key.clone(), value).is_some() {
                    return Err(de::Error::custom(format!("duplicate node ID {key}")));
                }
            }
            Ok(nodes)
        }
    }
    deserializer.deserialize_map(Nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../../../source-assets/district-map/district.json");

    #[test]
    fn real_map_and_axis_contract() {
        let map = Map::parse(SOURCE, "district.json").unwrap();
        assert!(map.buildings.iter().any(|b| b.id == "V-04"));
        let source = [12.5, -20.25, 7.75];
        assert_eq!(map_to_world(source), Vec3::new(12.5, 7.75, 20.25));
        assert_eq!(world_to_map(map_to_world(source)), source);
    }

    #[test]
    fn errors_locate_source_and_refuse_invalid_core_data() {
        let syntax = Map::parse("{\n\"version\": 7,", "broken.json")
            .unwrap_err()
            .to_string();
        assert!(
            syntax.contains("broken.json") && syntax.contains("line 2 column"),
            "{syntax}"
        );
        assert!(Map::parse(&format!("{SOURCE} false"), "trailing.json").is_err());
        for (path, value, expected) in [
            ("/buildings/0/height", Value::from("tall"), "V-01"),
            ("/buildings/0/height", Value::from(-1), "greater than zero"),
            (
                "/roads/0/nodes/0",
                Value::from("missing_node"),
                "missing node",
            ),
            (
                "/roads/0/kind",
                Value::from("teleport"),
                "unsupported value",
            ),
            ("/units/horizontal", Value::from("cm"), "expected m"),
            ("/version", Value::from(999), "expected 7"),
            (
                "/buildings/0/polygon",
                serde_json::json!([[0, 0], [1, 1], [2, 2]]),
                "area",
            ),
            (
                "/buildings/1/design/lightwell",
                serde_json::json!([[0, 0], [1, 0], [1, 1], [0, 1]]),
                "not contained",
            ),
        ] {
            let mut document: Value = serde_json::from_str(SOURCE).unwrap();
            *document.pointer_mut(path).unwrap() = value;
            let error = Map::parse(&document.to_string(), "fault.json").unwrap_err();
            assert_eq!(error.pointer, path, "{error}");
            let message = error.to_string();
            assert!(message.contains(expected), "{message}");
            assert!(
                message.contains("fault.json") && message.contains("actual="),
                "{message}"
            );
        }
        let duplicate = SOURCE.replacen(
            "\"nodes\": {",
            "\"nodes\": {\"duplicate\":[0,0,0],\"duplicate\":[1,1,1],",
            1,
        );
        assert!(
            Map::parse(&duplicate, "duplicate.json")
                .unwrap_err()
                .cause
                .contains("duplicate node ID")
        );
    }
}
