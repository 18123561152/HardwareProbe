use hardware_probe::SystemHardwareProbe;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = SystemHardwareProbe::new().inspect().await?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}
