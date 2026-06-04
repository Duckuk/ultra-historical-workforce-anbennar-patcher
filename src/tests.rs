use std::io::{BufWriter, Write};

use crate::paradox_data::ParadoxNode;

#[test]
fn node_display_test() -> Result<(), Box<dyn std::error::Error>> {
    let test_path = "test_123.txt";

    let nodes = ParadoxNode::from_file("03_mines.txt");
    let f = std::fs::File::create_new(test_path)?;
    let mut writer = BufWriter::new(f);
    for node in &nodes {
        writeln!(&mut writer, "{}", node)?;
    }
    drop(writer);

    let new_nodes = ParadoxNode::from_file(test_path);

    assert_eq!(nodes, new_nodes);

    std::fs::remove_file(test_path)?;

    Ok(())
}
