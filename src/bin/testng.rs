use std::time::Duration;

use anna_client_tokio::{config::Config, nodes::clientng::ClientNode, topics::RoutingThread};

fn set_up_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    if let Err(err) = set_up_logger() {
        eprintln!(
            "{:?}",
            eyre::Error::new(err).wrap_err("failed to set up logger")
        );
    }

    let config: Config = argh::from_env();
    log::info!("Loaded config from command line args: {:?}", config);

    let routing_threads: Vec<_> = (0..config.routing_threads)
        .map(|i| RoutingThread::new(i))
        .collect();

    let timeout = Duration::from_secs(10);
    let mut client = ClientNode::new(
        format!("client-{}", uuid::Uuid::new_v4()),
        0,
        routing_threads,
        timeout,
    )?;

    client.put_lww("foo".into(), "bar".into()).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let value = String::from_utf8(client.get_lww("foo".into()).await?)?;
    log::info!("Successfully get foo: {}", value);

    Ok(())
}