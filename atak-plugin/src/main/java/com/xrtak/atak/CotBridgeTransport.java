package com.xrtak.atak;

import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.InetAddress;
import java.nio.charset.StandardCharsets;

/**
 * Transport abstraction for outbound/inbound CoT XML frames (DIP).
 */
public interface CotBridgeTransport {
    void sendCotXml(String xmlPayload);

    final class Udp implements CotBridgeTransport {
        private final String host;
        private final int port;

        public Udp(String host, int port) {
            this.host = host;
            this.port = port;
        }

        @Override
        public void sendCotXml(final String xmlPayload) {
            new Thread(() -> {
                try (DatagramSocket socket = new DatagramSocket()) {
                    byte[] bytes = xmlPayload.getBytes(StandardCharsets.UTF_8);
                    InetAddress addr = InetAddress.getByName(host);
                    DatagramPacket pkt = new DatagramPacket(bytes, bytes.length, addr, port);
                    socket.send(pkt);
                } catch (Exception ignored) {
                }
            }).start();
        }
    }
}
