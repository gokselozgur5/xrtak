package com.xrtak.atak;

import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.InetAddress;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.ThreadFactory;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * Transport abstraction for outbound/inbound CoT XML frames (DIP).
 */
public interface CotBridgeTransport {
    void sendCotXml(String xmlPayload);
    void startInboundListener(InboundListener listener);
    void stopInboundListener();

    interface InboundListener {
        void onCotXmlReceived(String xmlPayload);
    }

    final class Udp implements CotBridgeTransport {
        private final String host;
        private final int port;
        private final int inboundPort;
        private final ExecutorService sendExecutor;
        private volatile boolean running;
        private DatagramSocket inboundSocket;
        private Thread inboundThread;

        public Udp(String host, int port) {
            this(host, port, 4243);
        }

        public Udp(String host, int port, int inboundPort) {
            this.host = host;
            this.port = port;
            this.inboundPort = inboundPort;
            this.sendExecutor = Executors.newFixedThreadPool(2, new ThreadFactory() {
                private final AtomicInteger count = new AtomicInteger(1);
                @Override
                public Thread newThread(Runnable r) {
                    Thread t = new Thread(r, "xrtak-cot-send-" + count.getAndIncrement());
                    t.setDaemon(true);
                    return t;
                }
            });
        }

        public String getHost() { return host; }
        public int getPort() { return port; }
        public int getInboundPort() { return inboundPort; }

        @Override
        public void sendCotXml(final String xmlPayload) {
            if (sendExecutor.isShutdown()) {
                return;
            }
            sendExecutor.submit(() -> {
                try (DatagramSocket socket = new DatagramSocket()) {
                    byte[] bytes = xmlPayload.getBytes(StandardCharsets.UTF_8);
                    InetAddress addr = InetAddress.getByName(host);
                    DatagramPacket pkt = new DatagramPacket(bytes, bytes.length, addr, port);
                    socket.send(pkt);
                } catch (Exception ignored) {
                }
            });
        }

        @Override
        public synchronized void startInboundListener(InboundListener listener) {
            if (running) return;
            running = true;
            inboundThread = new Thread(() -> {
                try {
                    inboundSocket = new DatagramSocket(inboundPort);
                    byte[] buf = new byte[65535];
                    while (running) {
                        DatagramPacket pkt = new DatagramPacket(buf, buf.length);
                        inboundSocket.receive(pkt);
                        String xml = new String(pkt.getData(), pkt.getOffset(), pkt.getLength(), StandardCharsets.UTF_8);
                        if (listener != null) {
                            listener.onCotXmlReceived(xml);
                        }
                    }
                } catch (Exception ignored) {
                }
            }, "xrtak-cot-inbound");
            inboundThread.setDaemon(true);
            inboundThread.start();
        }

        @Override
        public synchronized void stopInboundListener() {
            running = false;
            if (inboundSocket != null && !inboundSocket.isClosed()) {
                inboundSocket.close();
            }
            if (inboundThread != null) {
                inboundThread.interrupt();
            }
            sendExecutor.shutdown();
            try {
                if (!sendExecutor.awaitTermination(500, TimeUnit.MILLISECONDS)) {
                    sendExecutor.shutdownNow();
                }
            } catch (InterruptedException e) {
                sendExecutor.shutdownNow();
                Thread.currentThread().interrupt();
            }
        }
    }
}

