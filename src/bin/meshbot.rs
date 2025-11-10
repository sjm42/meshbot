// meshbot.rs

use clap::Parser;
use meshtastic::api::StreamApi;
use meshtastic::protobufs::{
    from_radio, mesh_packet, to_radio, Data, FromRadio, MeshPacket, PortNum,
};

use meshbot::*;

// Found help for sendig here:
// https://github.com/meshtastic/rust/issues/78

// Also, the supposedly simpler send_text() method would be nice, but...?
// https://github.com/meshtastic/rust/issues/63#issuecomment-3415187743

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut opts = OptsCommon::parse();
    opts.finish()?;
    opts.start_pgm(env!("CARGO_BIN_NAME"));
    debug!("Runtime config:\n{opts:#?}");

    let stream_api = StreamApi::new();
    let tcp_stream = meshtastic::utils::stream::build_tcp_stream(opts.address.clone()).await?;
    let (mut decoded_listener, stream_api) = stream_api.connect(tcp_stream).await;
    let config_id = meshtastic::utils::generate_rand_id();
    let mut stream_api = stream_api.configure(config_id).await?;

    while let Some(decoded) = decoded_listener.recv().await {
        debug!("Received: {decoded:?}");
        if let FromRadio {
            id: _id,
            payload_variant: Some(from_radio::PayloadVariant::Packet(meshpacket)),
        } = decoded
        {
            let mp = meshpacket.clone();
            if let Some(mesh_packet::PayloadVariant::Decoded(data)) = mp.payload_variant
                && !meshpacket.via_mqtt
            {
                info!("Got non-MQTT meshpacket: {meshpacket:?}");
                if data.portnum == PortNum::TextMessageApp as i32 {
                    debug!("*** Got MSG data: {data:?}");
                    let msg = String::from_utf8_lossy(&data.payload);
                    info!("Got MSG: \"{msg}\"");

                    if msg == "!ping" {
                        // Create a text message data payload
                        let data = Data {
                            portnum: PortNum::TextMessageApp as i32,
                            payload: "pong!".as_bytes().to_vec(),
                            want_response: false,
                            ..Default::default()
                        };

                        // Create the payload variant, mesh packet for broadcast
                        let payload_variant = Some(to_radio::PayloadVariant::Packet(MeshPacket {
                            to: 0xffffffff, // Broadcast address
                            from: 0,        // Will be filled by the device
                            channel: 0,
                            id: 0, // Will be assigned by the device
                            priority: mesh_packet::Priority::Default as i32,
                            payload_variant: Some(mesh_packet::PayloadVariant::Decoded(data)),
                            ..Default::default()
                        }));

                        // Send using the stream API's send_to_radio_packet method
                        info!("Attempting to send packet to Meshtastic radio...");

                        match stream_api.send_to_radio_packet(payload_variant).await {
                            Ok(_) => info!("Successfully sent to Meshtastic"),
                            Err(e) => error!("Failed to send to Meshtastic: {e}"),
                        }
                    }
                }
            }
        }
    }
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}
// EOF
