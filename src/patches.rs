use std::path::Path;

use lazy_regex::regex_switch;

use crate::paradox_data::{Employment, ParadoxNode, ParadoxValue};

// Override auto-generation won't touch these PMs because I've made my own overrides for them
const MINES_MANUAL_OVERRIDE_LIST: [&str; 5] = [
    "pm_automata_loggers",
    "pm_automata_miners",
    "pm_automata_foremen",
    "pm_transmute_to_gold_building_lead_mine",
    "pm_pathway_aurification_building_lead_mine",
];
const INDUSTRY_MANUAL_OVERRIDE_LIST: [&str; 2] = [
    //"pm_chronomantic_food_preservatives",
    "pm_automata_dough_processors",
    "pm_automata_bakers",
];
const MISC_RESOURCE_EXCLUSION_LIST: [&str; 6] = [
    "pm_basic_sea_floor_harvesting",
    "pm_damestear_scrying",
    "pm_diving_bells",
    "pm_harvester_mechs",
    "pm_water_breathing_potions",
    "pm_automata_crawlers",
];

// One-size-fits-all rule for automata PMs because I'm lazy but also don't want the auto-generation to make them useless
const AUTOMATA_DEFAULT_TARGET: Employment = Employment {
    engineers: 0,
    labourers: -2000,
    machinists: 0,
    shopkeepers: 0,
    farmers: 0,
    mages: 0,
    bureaucrats: 0,
    aristocrats: 0,
    capitalists: 0,
    soldiers: 0,
    officers: 0,
};
const AUTOMATA_ADVANCED_DEFAULT_TARGET: Employment = Employment {
    engineers: 0,
    labourers: -2000,
    machinists: -1000,
    shopkeepers: 0,
    farmers: 0,
    mages: 0,
    bureaucrats: 0,
    aristocrats: 0,
    capitalists: 0,
    soldiers: 0,
    officers: 0,
};

/// Searches top-level list of nodes for a name that matches
fn find_node<'a>(vec: &'a [ParadoxNode], name: &str) -> Option<&'a ParadoxNode> {
    vec.iter().find(|node| node.name() == name)
}

/// Checks if top-level node has a level-scaled effect on employment
fn has_employment_effect(node: &ParadoxNode) -> bool {
    match node.get(&["building_modifiers", "level_scaled"]) {
        Some(ParadoxNode::Value {
            value: ParadoxValue::Container(v),
            ..
        }) => v.iter().any(|n| n.name().contains("building_employment")),
        _ => false,
    }
}

/// Creates a patch for the production method `name` that uses `relations` for the specific employment modifiers.
///
/// For example:
/// ```ignore
/// Employment {
///     engineers: 50,  
///     labourers: 1000,  
///     machinists: 450,  
///     ..
/// }
/// ```
/// would become
/// ```norust
/// {name} = {
///     building_modifiers = {
///         level_scaled = {
///             building_employment_engineers_add = 50
///             building_employment_laborers_add = 1000
///             building_employment_machinists_add = 450
///         }
///     }
/// }
/// ```
///
///
/// Note that "INJECT:" must be included by the dev in the name because I've made it that way teehee
///
/// TODO: make relation be an `Employment` instead?
fn create_employment_override(name: &str, employment_mod: Employment) -> ParadoxNode {
    use ParadoxValue::*;
    ParadoxNode::Value {
        name: name.to_string(),
        value: Container(vec![ParadoxNode::Value {
            name: "building_modifiers".to_string(),
            value: Container(vec![ParadoxNode::Value {
                name: "level_scaled".to_string(),
                value: employment_mod.into(),
            }]),
        }]),
    }
}

fn generate_override_to_patch(
    patch: &mut Vec<ParadoxNode>,
    original_node: &ParadoxNode,
    percent_reduction: f64,
) {
    let original_employment = Employment::from_pm(original_node);
    let target_employment = original_employment * (1.0 - percent_reduction);
    let employment_mod = target_employment - original_employment;

    patch.push(create_employment_override(
        &format!("INJECT:{}", original_node.name()),
        employment_mod,
    ));
}

/// Returns nodes that mention `name` anywhere inside them.
fn find_name_references<'a>(nodes: &'a [ParadoxNode], name: &str) -> Vec<&'a ParadoxNode> {
    let mut v: Vec<&'a ParadoxNode> = Vec::new();
    for node in nodes {
        if node.name() == name {
            v.push(node);
        }
        if let ParadoxNode::Value {
            value: ParadoxValue::Container(list),
            ..
        } = node
        {
            v.append(&mut find_name_references(list, name));
        }
    }

    v
}

/// Unfinished. Idea was to make automata pms negate the maximum number of laborers & machinists the primary pm could give
///
/// Would look cleaner to the user but I'm not sure if it'd actually have any gameplay effect
#[expect(unused)]
fn find_primary_pm_peak_employment(
    building_name: &str,
    building_nodes: &[ParadoxNode],
    pmg_nodes: &[ParadoxNode],
    pm_nodes: &[ParadoxNode],
) -> Option<Employment> {
    let base_pmg_name = match find_node(building_nodes, building_name) {
        Some(n) => match n.get_value(&["production_method_groups"]) {
            Some(ParadoxValue::Container(v)) => match v.get(0) {
                Some(ParadoxNode::End { name }) => name,
                _ => return None,
            },
            _ => return None,
        },
        None => return None,
    };

    let pms: Vec<&str> = match find_node(pmg_nodes, base_pmg_name) {
        Some(n) => match n.get_value(&["production_methods"]) {
            Some(ParadoxValue::Container(v)) => v.iter().map(|n| n.name()).collect(),
            _ => return None,
        },
        _ => return None,
    };

    todo!()
}

pub fn patch_mines<P1: AsRef<Path>, P2: AsRef<Path>>(
    anbennar_path: P1,
    uhw_path: P2,
) -> Vec<ParadoxNode> {
    let anbennar_mines_path = anbennar_path
        .as_ref()
        .join("common/production_methods/anb_03_mines.txt");
    let uhw_mines_path = uhw_path
        .as_ref()
        .join("common/production_methods/Ultra_Wo_mines.txt");

    let anbennar_mines = ParadoxNode::from_file(anbennar_mines_path);
    let uhw_mines = ParadoxNode::from_file(uhw_mines_path);

    let uhw_picks_and_shovels_value =
        find_node(&uhw_mines, "INJECT:pm_picks_and_shovels_building_coal_mine")
            .unwrap()
            .value()
            .unwrap();

    let uhw_atmospheric_engine_pump_value = find_node(
        &uhw_mines,
        "INJECT:pm_atmospheric_engine_pump_building_coal_mine",
    )
    .unwrap()
    .value()
    .unwrap();

    let uhw_condensing_engine_pump_value = find_node(
        &uhw_mines,
        "INJECT:pm_condensing_engine_pump_building_coal_mine",
    )
    .unwrap()
    .value()
    .unwrap();

    let uhw_diesel_pump_value = find_node(&uhw_mines, "INJECT:pm_diesel_pump_building_coal_mine")
        .unwrap()
        .value()
        .unwrap();

    let uhw_nitroglycerin_value =
        find_node(&uhw_mines, "INJECT:pm_nitroglycerin_building_coal_mine")
            .unwrap()
            .value()
            .unwrap();

    let uhw_dynamite_value = find_node(&uhw_mines, "INJECT:pm_dynamite_building_coal_mine")
        .unwrap()
        .value()
        .unwrap();

    let mut final_patch: Vec<ParadoxNode> = Vec::new();

    for pm_node in anbennar_mines
        .iter()
        .filter(|n| MINES_MANUAL_OVERRIDE_LIST.iter().all(|s| *s != n.name()))
    {
        regex_switch!(pm_node.name(),
            // Vanilla analogues
            r"^pm_picks_and_shovels_building_(?<name>\w+)$"       => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_picks_and_shovels_building_{name}"), value: uhw_picks_and_shovels_value.clone() }),
            r"^pm_atmospheric_engine_pump_building_(?<name>\w+)$" => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_atmospheric_engine_pump_building_{name}"), value: uhw_atmospheric_engine_pump_value.clone() }),
            r"^pm_condensing_engine_pump_building_(?<name>\w+)$"  => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_condensing_engine_pump_building_{name}"), value: uhw_condensing_engine_pump_value.clone() }),
            r"^pm_diesel_pump_building_(?<name>\w+)$"             => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_diesel_pump_building_{name}"), value: uhw_diesel_pump_value.clone() }),
            r"^pm_nitroglycerin_building_(?<name>\w+)$"           => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_nitroglycerin_building_{name}"), value: uhw_nitroglycerin_value.clone() }),
            r"^pm_dynamite_building_(?<name>\w+)$"                => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_dynamite_building_{name}"), value: uhw_dynamite_value.clone() }),
            // Dynamite upgrade
            r"^pm_subterrenes_(?<name>\w+)$"                      => final_patch.push(ParadoxNode::Value { name: format!("INJECT:pm_subterrenes_{name}"), value: uhw_dynamite_value.clone() }),
            // Half employment of what's left
            r"^(?<name>\w+)$" => if has_employment_effect(pm_node) { generate_override_to_patch(&mut final_patch, pm_node, 0.6); }
        );
    }

    final_patch
}

pub fn patch_construction<P: AsRef<Path>>(anbennar_path: P) -> Vec<ParadoxNode> {
    let anbennar_construction_path = anbennar_path
        .as_ref()
        .join("common/production_methods/anb_13_construction.txt");

    let anbennar_mines = ParadoxNode::from_file(anbennar_construction_path);

    let mut final_patch: Vec<ParadoxNode> = Vec::new();

    for pm_node in anbennar_mines
        .iter()
        .filter(|n| MINES_MANUAL_OVERRIDE_LIST.iter().all(|s| *s != n.name()))
    {
        if has_employment_effect(pm_node) {
            generate_override_to_patch(&mut final_patch, pm_node, 0.6);
        }
    }

    final_patch
}

pub fn patch_industry<P1: AsRef<Path>, P2: AsRef<Path>>(
    anbennar_path: P1,
    uhw_path: P2,
) -> Vec<ParadoxNode> {
    let anbennar_industry_path = anbennar_path
        .as_ref()
        .join("common/production_methods/anb_01_industry.txt");
    let uhw_industry_path = uhw_path
        .as_ref()
        .join("common/production_methods/Ultra_Wo_industry.txt");
    let anbennar_vanilla_industry_path = anbennar_path
        .as_ref()
        .join("common/production_methods/01_industry.txt");

    let anbennar_industry = ParadoxNode::from_file(anbennar_industry_path);
    let uhw_industry = ParadoxNode::from_file(uhw_industry_path);
    let vanilla_industry = ParadoxNode::from_file(anbennar_vanilla_industry_path);

    let vanilla_electric_sewing_machines_node =
        find_node(&vanilla_industry, "pm_electric_sewing_machines").unwrap();
    let uhw_electric_sewing_machines_node =
        find_node(&uhw_industry, "INJECT:pm_electric_sewing_machines").unwrap();

    let mut final_patch: Vec<ParadoxNode> = Vec::new();

    for pm_node in anbennar_industry
        .iter()
        .filter(|n| INDUSTRY_MANUAL_OVERRIDE_LIST.iter().all(|s| *s != n.name()))
    {
        regex_switch!(pm_node.name(),
            r"^pm_chronomantic_food_preservatives$" => {
                let original_employment = Employment::from_pm(pm_node);
                let target_employment = Employment::from_pm(vanilla_electric_sewing_machines_node) + Employment::from_pm(uhw_electric_sewing_machines_node);
                let employment_mod = target_employment - original_employment;
                final_patch.push(create_employment_override("INJECT:pm_chronomantic_food_preservatives", employment_mod));
            },
            r"^pm_automata_laborers\w+$" => {
                let original_employment = Employment::from_pm(pm_node);
                let employment_mod = AUTOMATA_DEFAULT_TARGET - original_employment;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod))
            },
            r"^pm_automata_machinists\w+$" => {
                let original_employment = Employment::from_pm(pm_node);
                let employment_mod = AUTOMATA_ADVANCED_DEFAULT_TARGET - original_employment;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod))
            },
            // Half the employment of what's left
            r"^(?<name>\w+)$" => if has_employment_effect(pm_node) { generate_override_to_patch(&mut final_patch, pm_node, 0.5); }
        );
    }

    final_patch
}

pub fn patch_plantations<P: AsRef<Path>>(anbennar_path: P) -> Vec<ParadoxNode> {
    let anbennar_plantation_path = anbennar_path
        .as_ref()
        .join("common/production_methods/anb_04_plantations.txt");

    let anbennar_plantations = ParadoxNode::from_file(anbennar_plantation_path);

    let mut final_patch: Vec<ParadoxNode> = Vec::new();

    for pm_node in anbennar_plantations.iter() {
        regex_switch!(pm_node.name(),
            r"^pm_genetically_transmuted_organisms\w+$" => {
                let original_employment = Employment::from_pm(pm_node);
                let mut employment_mod = Employment::default();
                employment_mod.machinists = 0 - original_employment.machinists / 2;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod))
            },
            r"^pm_chronoponics\w+$" => {
                let original_employment = Employment::from_pm(pm_node);
                let mut employment_mod = Employment::default();
                employment_mod.machinists = 0 - original_employment.machinists / 2;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod))
            },
        );
    }

    final_patch
}

pub fn patch_misc_resource<P1: AsRef<Path>, P2: AsRef<Path>>(
    anbennar_path: P1,
    uhw_path: P2,
) -> Vec<ParadoxNode> {
    let anbennar_misc_resource_path = anbennar_path
        .as_ref()
        .join("common/production_methods/anb_09_misc_resource.txt");
    let uhw_misc_resource_path = uhw_path
        .as_ref()
        .join("common/production_methods/Ultra_Wo_misc_resource.txt");
    let anbennar_vanilla_misc_resource_path = anbennar_path
        .as_ref()
        .join("common/production_methods/09_misc_resource.txt");

    let anbennar_misc_resource = ParadoxNode::from_file(anbennar_misc_resource_path);
    let uhw_misc_resource = ParadoxNode::from_file(uhw_misc_resource_path);
    let anbennar_vanilla_misc_resource =
        ParadoxNode::from_file(anbennar_vanilla_misc_resource_path);

    let mut final_patch: Vec<ParadoxNode> = Vec::new();

    for pm_node in anbennar_misc_resource
        .iter()
        .filter(|n| MISC_RESOURCE_EXCLUSION_LIST.iter().all(|s| *s != n.name()))
    {
        regex_switch!(pm_node.name(),
            r"^pm_shredder_mechs\w*$" => {
                let original_employment = Employment::from_pm(pm_node);
                let mut employment_mod = AUTOMATA_ADVANCED_DEFAULT_TARGET - original_employment;
                employment_mod.engineers += 50;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod));
            },
            r"^(?<base_pm>pm_\w+)_cave_coral$" => 'out: {
                let vanilla_employment = {
                    let Some(node) = find_node(&anbennar_vanilla_misc_resource, base_pm) else { break 'out; };
                    Employment::from_pm(node)
                };
                let uhw_employment_mod = {
                    let Some(node) = find_node(&uhw_misc_resource, &format!("INJECT:{base_pm}")) else { break 'out; };
                    Employment::from_pm(node)
                };
                let target_employment = vanilla_employment + uhw_employment_mod;
                let employment_mod = target_employment - Employment::from_pm(pm_node);
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod));
            },
            r"^pm_genetically_transmuted_organisms\w+$" => {
                let original_employment = Employment::from_pm(pm_node);
                let mut employment_mod = Employment::default();
                employment_mod.machinists = 0 - original_employment.machinists / 2;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod))
            },
            r"^pm_chronoponics\w+$" => {
                let original_employment = Employment::from_pm(pm_node);
                let mut employment_mod = Employment::default();
                employment_mod.machinists = 0 - original_employment.machinists / 2;
                final_patch.push(create_employment_override(&format!("INJECT:{}", pm_node.name()), employment_mod))
            },
            // Half the employment of what's left
            r"^\w+$" => if has_employment_effect(pm_node) { generate_override_to_patch(&mut final_patch, pm_node, 0.2); }
        );
    }

    final_patch
}
