use std::{
    collections::HashMap,
    fmt::Display,
    fs::read,
    io::{BufRead, Cursor},
};

use lazy_regex::{
    bytes_regex_replace_all, regex_captures, regex_is_match, regex_replace_all, regex_switch,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ParadoxNode {
    Value {
        name: String,
        value: ParadoxValue,
    },
    End {
        name: String,
    },
    //Deprecated
    #[expect(unused)]
    Container {
        name: String,
        children: Vec<ParadoxNode>,
    },
}

impl ParadoxNode {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Vec<Self> {
        let path = path.as_ref();
        let text_data = {
            let mut buf = read(path).unwrap();
            if buf.starts_with(b"\xEF\xbb\xbf") {
                buf = buf[3..].to_vec();
            }

            // Remove comments
            buf = bytes_regex_replace_all!(r"#.*(\n?)", &buf, b"\n").to_vec();
            // Ensure there are new lines around brackets because my parser is dumb and line-based </3
            buf = bytes_regex_replace_all!(r"= \{", &buf, b"= {\n").to_vec();
            buf = bytes_regex_replace_all!(r"\}", &buf, b"\n}").to_vec();

            buf
        };

        let mut reader = Cursor::new(&text_data);
        let mut vec = Vec::new();

        loop {
            let line_start_pos = reader.position();
            let mut buf = String::new();
            if let Ok(0) = reader.read_line(&mut buf) {
                break;
            }

            if regex_is_match!(r"^.* = \{$", &buf.trim()) {
                let mut cursor = Cursor::new(&text_data);
                cursor.set_position(line_start_pos);
                let (new_pos, object) = read_object(cursor);
                reader.set_position(new_pos);
                vec.push(object);
            }
        }

        vec
    }

    pub fn name(&self) -> &str {
        use self::*;
        match self {
            ParadoxNode::Container { name, .. } => name,
            ParadoxNode::Value { name, .. } => name,
            ParadoxNode::End { name } => name,
        }
    }

    pub fn value(&self) -> Option<&ParadoxValue> {
        use self::*;
        match self {
            ParadoxNode::Value { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn child(&self, name: &str) -> Option<&Self> {
        use self::*;
        match self {
            ParadoxNode::Value {
                value: ParadoxValue::Container(nodes),
                ..
            } => nodes.iter().find(|node| node.name() == name),
            _ => None,
        }
    }

    pub fn get(&self, values: &[&str]) -> Option<&ParadoxNode> {
        if values.len() == 0 {
            return Some(self);
        }

        match self.child(values[0]) {
            Some(child) => child.get(&values[1..]),
            None => None,
        }
    }

    pub fn get_value(&self, values: &[&str]) -> Option<&ParadoxValue> {
        if values.len() == 0 {
            return self.value();
        }

        match self.child(values[0]) {
            Some(child) => child.get_value(&values[1..]),
            None => None,
        }
    }
}

impl Display for ParadoxNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::*;

        match self {
            ParadoxNode::Container { name, children } => {
                writeln!(f, "{name} = {{")?;
                for node in children {
                    let buf = format!("{node}");
                    writeln!(
                        f,
                        "{}",
                        regex_replace_all!(r"(?<body>.*\n?)", &buf, |_, body| format!(
                            "\t{}",
                            body
                        ))
                    )?;
                }
                write!(f, "}}")?;
            }
            ParadoxNode::Value { name, value } => {
                write!(f, "{name} = {value}")?;
            }
            ParadoxNode::End { name } => writeln!(f, "{name}")?,
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParadoxValue {
    String(String),
    Integer(i64),
    Float(f64),
    Other(String),
    Container(Vec<ParadoxNode>),
}

impl From<&str> for ParadoxValue {
    fn from(value: &str) -> Self {
        if let Ok(i) = value.parse::<i64>() {
            ParadoxValue::Integer(i)
        } else if let Ok(f) = value.parse::<f64>() {
            ParadoxValue::Float(f)
        } else if regex_is_match!(r#"^".*"$"#, value) {
            ParadoxValue::String(value[1..value.len() - 1].to_string())
        } else {
            ParadoxValue::Other(value.to_string())
        }
    }
}

impl From<Employment> for ParadoxValue {
    fn from(value: Employment) -> Self {
        let mut list = Vec::new();

        let Employment {
            engineers,
            labourers,
            machinists,
            shopkeepers,
            farmers,
            mages,
            bureaucrats,
            aristocrats,
            capitalists,
            soldiers,
            officers,
        } = value;

        if engineers != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_engineers_add".into(),
                value: ParadoxValue::Integer(engineers),
            });
        }

        if labourers != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_laborers_add".into(),
                value: ParadoxValue::Integer(labourers),
            });
        }

        if machinists != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_machinists_add".into(),
                value: ParadoxValue::Integer(machinists),
            });
        }

        if shopkeepers != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_shopkeepers_add".into(),
                value: ParadoxValue::Integer(shopkeepers),
            });
        }

        if farmers != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_farmers_add".into(),
                value: ParadoxValue::Integer(farmers),
            });
        }

        if mages != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_mages_add".into(),
                value: ParadoxValue::Integer(mages),
            });
        }

        if bureaucrats != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_bureaucrats_add".into(),
                value: ParadoxValue::Integer(bureaucrats),
            });
        }

        if aristocrats != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_aristocrats_add".into(),
                value: ParadoxValue::Integer(aristocrats),
            });
        }

        if capitalists != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_capitalists_add".into(),
                value: ParadoxValue::Integer(capitalists),
            });
        }

        if soldiers != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_soldiers_add".into(),
                value: ParadoxValue::Integer(soldiers),
            });
        }

        if officers != 0 {
            list.push(ParadoxNode::Value {
                name: "building_employment_officers_add".into(),
                value: ParadoxValue::Integer(officers),
            });
        }

        ParadoxValue::Container(list)
    }
}

impl From<i64> for ParadoxValue {
    fn from(value: i64) -> Self {
        ParadoxValue::Integer(value)
    }
}

impl Display for ParadoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::*;

        match self {
            ParadoxValue::String(s) => write!(f, "\"{s}\""),
            ParadoxValue::Integer(i) => write!(f, "{i}"),
            ParadoxValue::Float(float) => write!(f, "{float}"),
            ParadoxValue::Other(v) => write!(f, "{v}"),
            ParadoxValue::Container(elements) => {
                writeln!(f, "{{")?;
                for node in elements {
                    let buf = format!("{node}");
                    writeln!(
                        f,
                        "{}",
                        regex_replace_all!(r"(?<body>.*\n?)", &buf, |_, body| format!(
                            "\t{}",
                            body
                        ))
                    )?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Employment {
    pub engineers: i64,
    pub labourers: i64,
    pub machinists: i64,
    pub shopkeepers: i64,
    pub farmers: i64,
    pub mages: i64,
    pub bureaucrats: i64,
    pub aristocrats: i64,
    pub capitalists: i64,
    pub soldiers: i64,
    pub officers: i64,
}

impl Employment {
    pub fn from_pm(pm_node: &ParadoxNode) -> Self {
        let engineers = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_engineers_add",
        ]) {
            *i
        } else {
            0
        };

        let labourers = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_laborers_add",
        ]) {
            *i
        } else {
            0
        };

        let machinists = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_machinists_add",
        ]) {
            *i
        } else {
            0
        };

        let shopkeepers = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_shopkeepers_add",
        ]) {
            *i
        } else {
            0
        };

        let farmers = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_farmers_add",
        ]) {
            *i
        } else {
            0
        };

        let mages = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_mages_add",
        ]) {
            *i
        } else {
            0
        };

        let bureaucrats = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_bureaucrats_add",
        ]) {
            *i
        } else {
            0
        };

        let aristocrats = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_aristocrats_add",
        ]) {
            *i
        } else {
            0
        };

        let capitalists = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_capitalists_add",
        ]) {
            *i
        } else {
            0
        };

        let soldiers = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_soldiers_add",
        ]) {
            *i
        } else {
            0
        };

        let officers = if let Some(ParadoxValue::Integer(i)) = pm_node.get_value(&[
            "building_modifiers",
            "level_scaled",
            "building_employment_officers_add",
        ]) {
            *i
        } else {
            0
        };

        Self {
            engineers,
            labourers,
            machinists,
            shopkeepers,
            farmers,
            mages,
            bureaucrats,
            aristocrats,
            capitalists,
            soldiers,
            officers,
        }
    }
}

impl std::ops::Add for Employment {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            engineers: self.engineers + rhs.engineers,
            labourers: self.labourers + rhs.labourers,
            machinists: self.machinists + rhs.machinists,
            shopkeepers: self.shopkeepers + rhs.shopkeepers,
            farmers: self.farmers + rhs.farmers,
            mages: self.mages + rhs.mages,
            bureaucrats: self.bureaucrats + rhs.bureaucrats,
            aristocrats: self.aristocrats + rhs.aristocrats,
            capitalists: self.capitalists + rhs.capitalists,
            soldiers: self.soldiers + rhs.soldiers,
            officers: self.officers + rhs.officers,
        }
    }
}

impl std::ops::Sub for Employment {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            engineers: self.engineers - rhs.engineers,
            labourers: self.labourers - rhs.labourers,
            machinists: self.machinists - rhs.machinists,
            shopkeepers: self.shopkeepers - rhs.shopkeepers,
            farmers: self.farmers - rhs.farmers,
            mages: self.mages - rhs.mages,
            bureaucrats: self.bureaucrats - rhs.bureaucrats,
            aristocrats: self.aristocrats - rhs.aristocrats,
            capitalists: self.capitalists - rhs.capitalists,
            soldiers: self.soldiers - rhs.soldiers,
            officers: self.officers - rhs.officers,
        }
    }
}

impl std::ops::Mul<f64> for Employment {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            engineers: f64::round(self.engineers as f64 * rhs) as i64,
            labourers: f64::round(self.labourers as f64 * rhs) as i64,
            machinists: f64::round(self.machinists as f64 * rhs) as i64,
            shopkeepers: f64::round(self.shopkeepers as f64 * rhs) as i64,
            farmers: f64::round(self.farmers as f64 * rhs) as i64,
            mages: f64::round(self.mages as f64 * rhs) as i64,
            bureaucrats: f64::round(self.bureaucrats as f64 * rhs) as i64,
            aristocrats: f64::round(self.aristocrats as f64 * rhs) as i64,
            capitalists: f64::round(self.capitalists as f64 * rhs) as i64,
            soldiers: f64::round(self.soldiers as f64 * rhs) as i64,
            officers: f64::round(self.officers as f64 * rhs) as i64,
        }
    }
}

impl Into<HashMap<String, ParadoxValue>> for Employment {
    fn into(self) -> HashMap<String, ParadoxValue> {
        let mut map = HashMap::new();

        if self.engineers != 0 {
            map.insert(
                "building_employment_engineers_add".to_string(),
                self.engineers.into(),
            );
        }

        if self.labourers != 0 {
            map.insert(
                "building_employment_laborers_add".to_string(),
                self.labourers.into(),
            );
        }

        if self.machinists != 0 {
            map.insert(
                "building_employment_machinists_add".to_string(),
                self.machinists.into(),
            );
        }

        if self.shopkeepers != 0 {
            map.insert(
                "building_employment_shopkeepers_add".to_string(),
                self.shopkeepers.into(),
            );
        }

        if self.farmers != 0 {
            map.insert(
                "building_employment_farmers_add".to_string(),
                self.farmers.into(),
            );
        }

        if self.mages != 0 {
            map.insert(
                "building_employment_mages_add".to_string(),
                self.mages.into(),
            );
        }

        map
    }
}

#[expect(unreachable_code)]
fn read_object(mut reader: Cursor<&Vec<u8>>) -> (u64, ParadoxNode) {
    let mut children: Vec<ParadoxNode> = Vec::new();
    let name = {
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        regex_captures!(r"^(.*) = \{", &buf.trim())
            .unwrap()
            .1
            .to_owned()
    };

    'main_loop: loop {
        let line_start_pos = reader.position();
        let mut buf = String::new();
        if let Ok(0) = reader.read_line(&mut buf) {
            break;
        }

        regex_switch!(&buf.trim(),
            // Container
            r"^.* = \{$" => {
                let mut cursor = Cursor::new(*reader.get_ref());
                cursor.set_position(line_start_pos);
                let (new_pos, object) = read_object(cursor);
                reader.set_position(new_pos);
                children.push(object);
            },

            // Value
            r#"^(?<name>.*) = (?<value>.*)$"# => children.push(ParadoxNode::Value { name: name.to_string(), value: ParadoxValue::from(value) }),

            // End of container
            r"^\}$" => {
                break 'main_loop;
            },

            // End node
            r"^(?<x>\w+)$" => children.push(ParadoxNode::End { name: x.to_string() })
        );
    }

    (
        reader.position(),
        ParadoxNode::Value {
            name,
            value: ParadoxValue::Container(children),
        },
    )
}
