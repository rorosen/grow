use api::gen::grow::{
    measurement_service_client::MeasurementServiceClient, AirMeasurement, AirMeasurements,
    LightMeasurement, LightMeasurements,
};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MeasurementServiceClient::connect("http://[::1]:10001").await?;

    let measure_time = Utc::now().timestamp_millis();
    let air_request = tonic::Request::new(AirMeasurements {
        measure_time: measure_time.clone(),
        left: Some(AirMeasurement {
            temperature: 12.12,
            humidity: 13.13,
            pressure: 14.14,
            resistance: 15.15,
        }),
        right: Some(AirMeasurement {
            temperature: 1.1,
            humidity: 2.2,
            pressure: 3.3,
            resistance: 4.4,
        }),
    });

    let light_request = tonic::Request::new(LightMeasurements {
        measure_time,
        left: Some(LightMeasurement { lux: 69.69 }),
        right: Some(LightMeasurement { lux: 0.1 }),
    });

    client.create_air_measurements(air_request).await?;
    client.create_light_measurements(light_request).await?;

    Ok(())
}
