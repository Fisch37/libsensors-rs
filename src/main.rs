use libsensors_rs::LibSensors;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = LibSensors::init().unwrap();
    for chip in lib.get_chips().unwrap() {
        let chip = chip?;
        println!("C: {} ({:?})", chip.get_name_raw()?.unwrap().to_str()?, chip.get_prefix());
        for feature in chip.get_features()? {
            let feature = feature?;
            println!("  F: {} ({:?})", feature.get_label().unwrap(), feature.get_name());
            for subfeature in feature.get_subfeatures()? {
                println!("    {:?}", subfeature.get_name().unwrap())
            }
        }
    }
    Ok(())
}
