use std::time::SystemTime;

use grow_utils::api::grow::{
    measurement_service_client::MeasurementServiceClient, AirMeasurement, AirSample,
    LightMeasurement, LightSample, WaterLevelMeasurement,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MeasurementServiceClient::connect("http://[::1]:10001").await?;

    let air_request = tonic::Request::new(AirMeasurement {
        measure_time: Some(SystemTime::now().into()),
        left: Some(AirSample {
            temperature: 12.12,
            humidity: 13.13,
            pressure: 14.14,
            resistance: 15.15,
        }),
        right: Some(AirSample {
            temperature: 1.1,
            humidity: 2.2,
            pressure: 3.3,
            resistance: 4.4,
        }),
    });

    let light_request = tonic::Request::new(LightMeasurement {
        measure_time: Some(SystemTime::now().into()),
        left: Some(LightSample { lux: 69.69 }),
        right: None,
    });

    let water_level_request = tonic::Request::new(WaterLevelMeasurement {
        measure_time: Some(SystemTime::now().into()),
        distance: 666,
    });

    client.create_air_measurement(air_request).await?;
    client.create_light_measurement(light_request).await?;
    client
        .create_water_level_measurement(water_level_request)
        .await?;

    Ok(())
}
