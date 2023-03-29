mod broker;
use std::time::Duration;

use broker::*;

use rumqttc::*;
use tokio::{task, time};

#[tokio::test]
async fn validates_topic() {
    let very_long_topic = String::from_utf8(vec![97; 65535 + 1]).unwrap();
    let not_very_long_topic = String::from_utf8(vec![97; 65535]).unwrap();

    let invalid_topics = vec!["", "\0", "hello/+", "hi/#", &very_long_topic];
    let valid_topics = vec!["a", "\\0", "hello", &not_very_long_topic];
    task::spawn(async move {
        let _broker = Broker::new(1881, 3).await;
        time::sleep(Duration::from_secs(10)).await;
    });
    time::sleep(Duration::from_secs(1)).await;

    let options = MqttOptions::new("dummy", "127.0.0.1", 1881);

    let (client, mut eventloop) = AsyncClient::new(options, 5);
    // Start the client eventloop
    task::spawn(async move {
        loop {
            eventloop.poll().await.unwrap();
        }
    });

    // Try publishing to invalid topics
    for invalid_topic in invalid_topics {
        match client
            .publish(invalid_topic, QoS::AtMostOnce, false, *b"")
            .await
        {
            Err(ClientError::InvalidTopic(_)) => (),
            Err(err) => panic!(
                "Expected `ClientError::InvalidTopic`. Got different error => {:?}",
                err
            ),
            Ok(_) => panic!(
                "Invalid topic '{}' was accepted by publish()",
                invalid_topic
            ),
        }
    }
    for valid_topic in valid_topics {
        if let Err(_) = client
            .publish(valid_topic, QoS::AtMostOnce, false, *b"")
            .await
        {
            panic!("Valid topic '{}' was rejected by publish()", valid_topic);
        }
    }
}

#[tokio::test]
async fn validates_filter() {
    let very_long_topic = String::from_utf8(vec![97; 65535 + 1]).unwrap();
    let invalid_filters = vec![
        "",
        "\0",
        "hello/++",
        "hi/#/there",
        "++",
        "##",
        &very_long_topic,
    ];

    let not_very_long_topic = String::from_utf8(vec![97; 65535]).unwrap();
    let valid_filters = vec![
        "a",
        "\\0",
        "hello/+",
        "hi/#",
        "+",
        "#",
        &not_very_long_topic,
    ];
    task::spawn(async move {
        let _broker = Broker::new(1882, 3).await;
        time::sleep(Duration::from_secs(10)).await;
    });
    time::sleep(Duration::from_secs(1)).await;

    let options = MqttOptions::new("dummy", "127.0.0.1", 1882);

    let (client, mut eventloop) = AsyncClient::new(options, 10);
    // Start the client eventloop
    task::spawn(async move {
        loop {
            eventloop.poll().await.unwrap();
        }
    });

    // Try subscribing to invalid filters
    for invalid_filter in &invalid_filters {
        match client.subscribe(*invalid_filter, QoS::AtMostOnce).await {
            Err(ClientError::InvalidFilter(_)) => (),
            Err(err) => panic!(
                "Expected `ClientError::InvalidFilter`. Got different error => {:?}",
                err
            ),
            Ok(_) => panic!(
                "Invalid filter '{}' was accepted by subscribe()",
                invalid_filter
            ),
        }
    }
    // Try subscribing to valid filters
    for valid_filter in &valid_filters {
        if let Err(err) = client.subscribe(*valid_filter, QoS::AtMostOnce).await {
            panic!(
                "Valid filter '{}' was rejected by subscribe(). Error => {:?}",
                valid_filter, err
            );
        }
    }

    // Try subsribing to invalid filters with subscribe_many
    let invalid_subscribe_filters = invalid_filters
        .into_iter()
        .map(|filter| SubscribeFilter::new(String::from(filter), QoS::AtMostOnce));

    match client.subscribe_many(invalid_subscribe_filters).await {
        Err(ClientError::InvalidFilter(_)) => (),
        Err(err) => panic!(
            "Expected `ClientError::InvalidFilter`. Got different error => {:?}",
            err
        ),
        Ok(_) => panic!("Invalid filter was accepted by subscribe_many()"),
    }

    // Try subsribing to valid filters with subscribe_many
    let valid_subscribe_filters = valid_filters
        .into_iter()
        .map(|filter| SubscribeFilter::new(String::from(filter), QoS::AtMostOnce));

    if let Err(err) = client.subscribe_many(valid_subscribe_filters).await {
        panic!(
            "Valid filter was rejected by subscribe_many(). Error => {:?}",
            err
        );
    }
}
