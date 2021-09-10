use deadpool_redis::{PoolConfig, Runtime};

#[tokio::test]
async fn redis_test() {
    let pool = deadpool_redis::Config {
        url: Some("redis://localhost".to_string()),
        connection: None,
        pool: Some(PoolConfig {
            max_size: 1,
            timeouts: Default::default(),
        })
    }
    .create_pool(Runtime::Tokio1)
    .unwrap();
    // let pool = redis::Client::open("redis://localhost".to_string()).unwrap();

    let stream_key = "test_stream";
    let group = "test_group";
    let consumer = "test_consumer";
    {
        let mut conn = pool.try_get().await.expect("conn");

        // del anything under the key in case previous test didn't clean up properly
        redis::Cmd::del(&stream_key[..])
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();

        // create consumer group
        redis::Cmd::xgroup_create_mkstream(&stream_key[..], &group[..], &["0"])
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();

        println!("created group");
    }
    {
        let mut conn = pool.try_get().await.expect("conn");

        // add message to stream
        redis::Cmd::xadd(&stream_key[..], "*", &[("stuff", "stuffer")][..])
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();

        println!("added message");
    }
    {
        let mut conn = pool.try_get().await.expect("conn");

        // async inf loop with timeout
        let _ = tokio::spawn(tokio::time::timeout(
            std::time::Duration::from_millis(500),
            async move {
                // start from the begenning
                let mut id = "0";
                loop {
                    let items = redis::Cmd::xread_options(
                        &[&stream_key[..]],
                        &[&id[..]],
                        &redis::streams::StreamReadOptions::default()
                            .block(0)
                            .count(10)
                            .group(&group[..], &consumer[..]),
                    )
                    .query_async::<_, redis::streams::StreamReadReply>(&mut conn)
                    .await
                    .unwrap();

                    let mut ctr = 0;
                    for redis::streams::StreamKey { ids, .. } in items.keys {
                        for redis::streams::StreamId { id: msg_id, .. } in ids {
                            redis::Cmd::xack(&stream_key[..], &group[..], &[msg_id])
                                .query_async::<_, ()>(&mut conn)
                                .await
                                .unwrap();
                            ctr += 1;
                        }
                    }
                    // if no items are returned, this means we've read everything availiable
                    // block for new items now
                    if ctr == 0 {
                        id = ">";
                    }
                }
            },
        ))
        .await
        .unwrap();

        println!("ended loop");
    }

    {
        let mut conn = pool.try_get().await.expect("conn");

        println!("got connection");

        let stream_msgs = redis::Cmd::xread_options(
            &[&stream_key[..]],
            &[">"],
            &redis::streams::StreamReadOptions::default()
                .count(1)
                .group(&group[..], &consumer[..]),
        )
        .query_async::<_, redis::streams::StreamReadReply>(&mut conn)
        .await
        .unwrap();

        // tracing::info!("got here: {:?}", stream_msgs);
        let stream_msgs = &stream_msgs.keys;
        // let stream_msgs = &stream_msgs.keys[0].ids;
        assert!(stream_msgs.is_empty());
        println!("checked for items");
    }
}
