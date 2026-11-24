package com.xrtak.atak;

import com.atakmap.android.maps.PointMapItem;
import com.atakmap.coremap.maps.coords.GeoPoint;
import java.util.Locale;

/**
 * Single-responsibility serializer converting ATAK PointMapItem state into ICD-01 CoT XML.
 */
public final class AtakItemSerializer {
    private AtakItemSerializer() {}

    /**
     * Escapes standard XML attribute characters (&, <, >, ", ').
     */
    public static String escapeXmlAttr(String val) {
        if (val == null || val.isEmpty()) {
            return "";
        }
        StringBuilder sb = new StringBuilder(val.length() + 16);
        for (int i = 0; i < val.length(); i++) {
            char c = val.charAt(i);
            switch (c) {
                case '&':
                    sb.append("&amp;");
                    break;
                case '<':
                    sb.append("&lt;");
                    break;
                case '>':
                    sb.append("&gt;");
                    break;
                case '"':
                    sb.append("&quot;");
                    break;
                case '\'':
                    sb.append("&apos;");
                    break;
                default:
                    sb.append(c);
                    break;
            }
        }
        return sb.toString();
    }

    public static String toCotXml(PointMapItem item) {
        GeoPoint pt = item.getPoint();
        String uid = escapeXmlAttr(item.getUID());
        String rawType = item.getType() != null ? item.getType() : "a-u-G";
        if ("self".equalsIgnoreCase(rawType)) {
            rawType = "a-f-G-U-C";
        }
        String type = escapeXmlAttr(rawType);
        String callsign = escapeXmlAttr(item.getMetaString("callsign", item.getUID()));
        double course = item.getMetaDouble("Speed.course", 0.0);
        double speed = item.getMetaDouble("Speed.value", 0.0);
        double hae = Double.isNaN(pt.getAltitude()) ? 0.0 : pt.getAltitude();

        return String.format(
            Locale.US,
            "<event version=\"2.0\" uid=\"%s\" type=\"%s\" how=\"m-g\">"
                + "<point lat=\"%.6f\" lon=\"%.6f\" hae=\"%.1f\" ce=\"10\" le=\"10\"/>"
                + "<detail><contact callsign=\"%s\"/><track course=\"%.1f\" speed=\"%.1f\"/></detail>"
                + "</event>",
            uid, type, pt.getLatitude(), pt.getLongitude(), hae, callsign, course, speed
        );
    }
}

