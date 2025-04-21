#[cfg(test)]
mod test {

    use axum::body::Body;
    use futures::SinkExt;
    use http_body_util::StreamBody;
    use opendal::Operator;
    use opendal::Result;
    use opendal::layers::LoggingLayer;
    use opendal::services;

    #[tokio::test]
    async fn test_opendal_0() -> Result<()> {
        // Pick a builder and configure it.
        let builder = services::Fs::default().root("./");

        // Init an operator
        let op = Operator::new(builder)?
            // Init with logging layer enabled.
            .layer(LoggingLayer::default())
            .finish();

        // Write data
        // op.write("hello.txt", "Hello, World!").await?;

        // Read data
        let bs = op.read("../Cargo.toml").await?;
        println!("{}", String::from_utf8_lossy(&bs.to_vec()));

        // read stream
        {

            let reader = op.reader("../Cargo.toml").await?;
            let stream = reader.into_stream(..).await?;


            let body_ds = Body::from("").into_data_stream();

            Body::new(StreamBody::new(stream).into()).into_data_stream();
            // Body::new(StreamBody::new(Body::from("")));
            // Body::try_from(stream.compat_mut());
        }

                // read stream
                {

                    let reader = op.reader("../Cargo.toml").await?;
                    let writer = op.writer("hello.txt").await?;
                    let stream = reader.into_bytes_stream(..).await?; 
                    let write_sink = writer.into_bytes_sink();
                    write_sink.send_all(&mut stream).await.unwrap();
        
        
                    let body_ds = Body::from("").into_data_stream();
        
                    Body::new(StreamBody::new(stream).into()).into_data_stream();
                    // Body::new(StreamBody::new(Body::from("")));
                    // Body::try_from(stream.compat_mut());
                }

        // Fetch metadata
        // let meta = op.stat("hello.txt").await?;
        // let mode = meta.mode();
        // let length = meta.content_length();

        // // Delete
        // op.delete("hello.txt").await?;

        Ok(())
    }


}

mod testt {
    use axum::body::Body;
    use opendal::{services, Operator};
    use http_body_util::StreamBody;
    use futures::StreamExt;


    async fn to_body() -> Body {

        let op = Operator::new(services::Fs::default().root("./")).unwrap().finish();
        let reader = op.reader("../Cargo.toml").await.unwrap();
        let stream = reader.into_stream(..).await.unwrap();

        let stream = stream
            .map(|res: Result<opendal::Buffer, opendal::Error>| 
                res
                .map(|val| val.to_bytes())
                .map_err(|e| axum::Error::new(format!("{:?}", e))))
            .boxed();

        // stream implements futures_core::stream::Stream so I expected this to work, but it doesn't:
        let body = Body::from_stream(stream);

        // I can build a StreamBody but then I fail to create a Body from it
        // let stream_body = Body::new(StreamBody::new(stream).into());

        // Now I am not able to convert the stream_body to a Body
        body
    }

    async fn to_body_2() -> Body {

        let op = Operator::new(services::Fs::default().root("./")).unwrap().finish();
        let reader = op.reader("../Cargo.toml").await.unwrap();
        let stream = reader.into_bytes_stream(..).await.unwrap();

        // let stream = stream
        //     .map(|res: Result<opendal::Buffer, opendal::Error>| 
        //         res
        //         .map(|val| val.to_bytes())
        //         .map_err(|e| axum::Error::new(format!("{:?}", e))))
        //     .boxed();

        // stream implements futures_core::stream::Stream so I expected this to work, but it doesn't:
        let body = Body::from_stream(stream);

        // I can build a StreamBody but then I fail to create a Body from it
        // let stream_body = Body::new(StreamBody::new(stream).into());

        // Now I am not able to convert the stream_body to a Body
        body
    }

}
