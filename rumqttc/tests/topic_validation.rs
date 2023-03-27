mod broker;
use std::time::Duration;

use broker::*;

use rumqttc::*;
use tokio::{task, time};

#[tokio::test]
async fn validates_topic() {
    let very_long_topic = String::from_utf8(vec![97; 65535 + 1]).unwrap();
    let invalid_topics = vec!["", "\0", "hello/+", "hi/#", &very_long_topic];
    task::spawn(async move {
        let _broker = Broker::new(1880, 3).await;
        time::sleep(Duration::from_secs(10)).await;
    });
    time::sleep(Duration::from_secs(1)).await;

    let options = MqttOptions::new("dummy", "127.0.0.1", 1880);

    let (client, mut eventloop) = AsyncClient::new(options, 5);
    // Start the client eventloop
    task::spawn(async move {
        loop {
            eventloop.poll().await.unwrap();
        }
    });

    // Try publishing to invalid topics
    for invalid_topic in invalid_topics {
        if let Ok(_) = client
            .publish(invalid_topic, QoS::AtMostOnce, false, *b"")
            .await
        {
            panic!(
                "Invalid topic '{}' was accepted by publish()",
                invalid_topic
            );
        }
    }
}
