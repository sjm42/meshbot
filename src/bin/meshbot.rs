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


const MESH_BROADCAST: u32 = u32::MAX;

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
            payload_variant: Some(from_radio::PayloadVariant::Packet(rx_packet)),
        } = decoded
        {
            let mp = rx_packet.clone();
            if let Some(mesh_packet::PayloadVariant::Decoded(rx_data)) = mp.payload_variant
                && !rx_packet.via_mqtt
            // we don't want MQTT packets here
            {
                info!("Received meshpacket: {rx_packet:?}");
                if rx_data.portnum == PortNum::TextMessageApp as i32 {
                    debug!("*** Got MSG data: {rx_data:?}");
                    let msg = String::from_utf8_lossy(&rx_data.payload);
                    info!("Got MSG: \"{msg}\"");

                    // responding to broadcast packets must be specifically enabled
                    if msg == "!ping" && (opts.broadcast || rx_packet.to != MESH_BROADCAST) {
                        let hop_count = rx_packet.hop_start - rx_packet.hop_limit;
                        let mut msg = format!("Pong, {hop_count} hops");

                        if hop_count == 0 {
                            msg.push_str(&format!(
                                ", SNR {:+.1}dB, RSSI {}dBm",
                                rx_packet.rx_snr, rx_packet.rx_rssi
                            ));
                        }
                        info!("Sending reply: \"{msg}\"");

                        // Create a text message data payload
                        let tx_data = Data {
                            portnum: PortNum::TextMessageApp as i32,
                            payload: msg.as_bytes().to_vec(),
                            want_response: true,
                            ..Default::default()
                        };

                        // Create the payload variant, mesh packet for broadcast
                        let tx_packet = Some(to_radio::PayloadVariant::Packet(MeshPacket {
                            channel: 0,
                            from: 0,            // Will be filled by the device
                            to: rx_packet.from, // do not send reply as broadcast but DM instead
                            id: 0,              // Will be assigned by the device
                            priority: mesh_packet::Priority::Default as i32,
                            hop_limit: 7,
                            want_ack: true,
                            payload_variant: Some(mesh_packet::PayloadVariant::Decoded(tx_data)),
                            ..Default::default()
                        }));

                        // Send using the stream API's send_to_radio_packet method
                        info!("Attempting to send packet: {tx_packet:?}");

                        match stream_api.send_to_radio_packet(tx_packet).await {
                            Ok(_) => info!("Successfully sent reply to mesh"),
                            Err(e) => error!("Failed to send: {e}"),
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
