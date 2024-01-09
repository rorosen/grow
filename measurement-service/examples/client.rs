use std::time::SystemTime;

use api::gen::grow::{
    measurement_service_client::MeasurementServiceClient, LightMeasurement, LightMeasurements,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MeasurementServiceClient::connect("http://[::1]:10001").await?;

    let request = tonic::Request::new(LightMeasurements {
        measure_time: Some(SystemTime::now().into()),
        left: Some(LightMeasurement { lux: 69.69 }),
        right: Some(LightMeasurement { lux: 0.1 }),
    });

    let response = client.create_light_measurements(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
