package com.xrtak.atak;

import android.content.Context;
import android.content.Intent;
import com.atakmap.android.cot.CotMapComponent;
import com.atakmap.android.maps.AbstractMapComponent;
import com.atakmap.android.maps.MapEvent;
import com.atakmap.android.maps.MapEventDispatcher;
import com.atakmap.android.maps.MapItem;
import com.atakmap.android.maps.MapView;
import com.atakmap.android.maps.PointMapItem;
import com.atakmap.coremap.cot.event.CotEvent;

/** Bidirectional ATAK-CIV 5.2 MapComponent bridging MapEventDispatcher and CotMapComponent. */
public class XrTakMapComponent extends AbstractMapComponent
        implements MapEventDispatcher.MapEventDispatchListener {
    private final CotBridgeTransport transport;

    public XrTakMapComponent() {
        this(new CotBridgeTransport.Udp("127.0.0.1", 4242, 4243));
    }

    public XrTakMapComponent(String targetHost, int targetPort, int listenPort) {
        this(new CotBridgeTransport.Udp(targetHost, targetPort, listenPort));
    }

    public XrTakMapComponent(CotBridgeTransport transport) {
        this.transport = transport;
    }

    public CotBridgeTransport getTransport() {
        return transport;
    }

    @Override
    public void onCreate(Context context, Intent intent, MapView view) {
        MapEventDispatcher d = view.getMapEventDispatcher();
        d.addMapEventListener(MapEvent.ITEM_ADDED, this);
        d.addMapEventListener(MapEvent.ITEM_REFRESH, this);
        d.addMapEventListener(MapEvent.ITEM_SHARED, this);
        transport.startInboundListener(xml -> dispatchInboundCotXml(xml, false));
    }

    @Override
    protected void onDestroyImpl(Context context, MapView view) {
        view.getMapEventDispatcher().removeMapEventListener(this);
        transport.stopInboundListener();
    }

    @Override
    public void onMapEvent(MapEvent event) {
        MapItem item = event.getItem();
        if (item instanceof PointMapItem) {
            transport.sendCotXml(AtakItemSerializer.toCotXml((PointMapItem) item));
        }
    }

    public void dispatchInboundCotXml(String cotXml, boolean broadcastExternal) {
        CotEvent ev = CotEvent.parse(cotXml);
        if (ev != null && ev.isValid()) {
            CotMapComponent.getInternalDispatcher().dispatch(ev);
            if (broadcastExternal) CotMapComponent.getExternalDispatcher().dispatch(ev);
        }
    }
}
