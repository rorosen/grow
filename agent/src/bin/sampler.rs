use grow_agent::manage::sample::{air::air_sensor::AirSensor, light::LightSensor};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    // let water = WaterLevelSensor::new(0x29).await?;
    let air_left = AirSensor::new(0x77).await;
    let air_right = AirSensor::new(0x76).await;
    let light_left = LightSensor::new(0x23).await;
    let light_right = LightSensor::new(0x5C).await;

    match air_left {
        Ok(mut s) => match s.measure(token.clone()).await {
            Ok(m) => println!("left air measurement: {m:?}"),
            Err(err) => println!("failed to take left air measurement: {err:#}"),
        },
        Err(err) => println!("failed to initialize left air sensor: {err:#}"),
    }

    match air_right {
        Ok(mut s) => match s.measure(token.clone()).await {
            Ok(m) => println!("right air measurement: {m:?}"),
            Err(err) => println!("failed to take right air measurement: {err:#}"),
        },
        Err(err) => println!("failed to initialize right air sensor: {err:#}"),
    }

    match light_left {
        Ok(mut s) => match s.measure(token.clone()).await {
            Ok(m) => println!("left light measurement: {m:?}"),
            Err(err) => println!("failed to take left light measurement: {err:#}"),
        },
        Err(err) => println!("failed to initialize left light sensor: {err:#}"),
    }

    match light_right {
        Ok(mut s) => match s.measure(token.clone()).await {
            Ok(m) => println!("right light measurement: {m:?}"),
            Err(err) => println!("failed to take right light measurement: {err:#}"),
        },
        Err(err) => println!("failed to initialize right light sensor: {err:#}"),
    }
}
