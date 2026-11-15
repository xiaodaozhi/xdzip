// Owned API harness; reference implementation code remains outside the product.
import java.io.*;
import java.nio.file.*;
import java.util.*;
import jp.gr.java_conf.dangan.util.lha.*;

public final class Reference {
    private static final class Bits {
        final ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        int value, count;
        void put(int value, int width) {
            for (int i = width - 1; i >= 0; i--) {
                this.value = (this.value << 1) | ((value >>> i) & 1);
                if (++count == 8) { bytes.write(this.value); count = 0; this.value = 0; }
            }
        }
        byte[] finish() { if (count != 0) put(0, 8 - count); return bytes.toByteArray(); }
    }
    private static void path(Bits output, DynamicHuffman tree, int symbol) {
        int[] path = new int[512]; int length = 0, node = tree.codeToNode(symbol);
        while (node != DynamicHuffman.ROOT) {
            int parent = tree.parentNode(node);
            path[length++] = tree.childNode(parent) == node ? 0 : 1;
            node = parent;
        }
        for (int i = length - 1; i >= 0; i--) output.put(path[i], 1);
    }
    private static byte[] unixSpaces() {
        Bits output = new Bits();
        DynamicHuffman codes = new DynamicHuffman(286), positions = new DynamicHuffman(128, 1);
        path(output, codes, 32); codes.update(32);
        int produced = 1, next = 64;
        for (int i = 0; i < 16; i++) {
            int length = i == 15 ? 255 : 256;
            path(output, codes, 285); codes.update(285); output.put(length + 253 - 285, 8);
            while (produced > next && next < 8192) { positions.addLeaf(next / 64); next += 64; }
            path(output, positions, 0);
            // Original Unix LH2 does not increment a singleton position leaf.
            if (positions.codeToNode(0) != DynamicHuffman.ROOT) positions.update(0);
            output.put(0, 6); produced += length;
        }
        return output.finish();
    }
    public static void main(String[] args) throws Exception {
        if (args.length == 2 && args[0].equals("encode-jlha-short")) {
            ByteArrayOutputStream packed = new ByteArrayOutputStream();
            PostLh2Encoder encoder = new PostLh2Encoder(packed);
            for (int i = 0; i < 64; i++) encoder.writeCode(i);
            int state = 0x6d2b79f5, produced = 64;
            for (int i = 0; i < 33000; i++) {
                state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
                int offset = (int) (Integer.toUnsignedLong(state) % Math.min(produced, 8192));
                encoder.writeCode(256); encoder.writeOffset(offset); produced += 3;
            }
            encoder.close(); Files.write(Path.of(args[1]), packed.toByteArray());
            return;
        }
        if (args.length == 2 && args[0].equals("encode-unix-spaces")) {
            byte[] payload = new byte[4096]; Arrays.fill(payload, (byte) 32); byte[] packed = unixSpaces();
            CRC16 crc = new CRC16(); crc.update(payload);
            LhaHeader header = new LhaHeader("hello.bin", new Date(1700000000000L));
            header.setHeaderLevel(2); header.setCompressMethod("-lh2-"); header.setOriginalSize(payload.length); header.setCompressedSize(packed.length); header.setCRC((int) crc.getValue());
            byte[] encoded = header.getBytes("UTF-8");
            try (OutputStream out = Files.newOutputStream(Path.of(args[1]))) { out.write(encoded); out.write(packed); out.write(0); }
            System.out.println(encoded.length + " " + packed.length + " " + payload.length + " " + crc.getValue());
            return;
        }
        if (args.length == 4 && args[0].equals("trace-lh2")) {
            int expected = Integer.parseInt(args[1]);
            PreLzssDecoder decoder = new PreLh2Decoder(Files.newInputStream(Path.of(args[2])));
            StringBuilder trace = new StringBuilder("[\n");
            int position = 0;
            try {
                while (position < expected) {
                    int code = decoder.readCode();
                    int offset = code < 256 ? -1 : decoder.readOffset();
                    trace.append(position == 0 ? "" : ",\n").append("[").append(position).append(",").append(code).append(",").append(offset).append("]");
                    position += code < 256 ? 1 : code - 256 + decoder.getThreshold();
                }
            } finally { decoder.close(); }
            if (position != expected) throw new IOException("Reference trace size mismatch");
            Files.writeString(Path.of(args[3]), trace.append("\n]\n").toString());
            return;
        }
        if (args.length == 3 && args[0].equals("decode")) {
            try (LhaInputStream input = new LhaInputStream(Files.newInputStream(Path.of(args[1])))) {
                LhaHeader header = input.getNextEntry();
                if (header == null) throw new IOException("Missing owned reference member");
                byte[] output = input.readAllBytes();
                if (input.getNextEntry() != null) throw new IOException("Unexpected extra reference member");
                Files.write(Path.of(args[2]), output);
            }
            return;
        }
        if ((args.length < 5 || args.length > 7) || !args[0].equals("encode")) throw new IllegalArgumentException("Invalid owned harness command");
        String method = "-" + args[1] + "-";
        int level = Integer.parseInt(args[2]);
        byte[] payload = Files.readAllBytes(Path.of(args[3]));
        if (payload.length > 2 * 1024 * 1024) throw new IOException("Owned fixture input limit");
        ByteArrayOutputStream packed = new ByteArrayOutputStream();
        try (OutputStream encoder = CompressMethod.connectEncoder(packed, method, LhaProperty.getProperties())) {
            encoder.write(payload);
        }
        CRC16 crc = new CRC16();
        crc.update(payload);
        LhaHeader header = new LhaHeader(args.length >= 6 ? args[5] : "hello.bin", new Date(1700000000000L));
        header.setHeaderLevel(level);
        header.setCompressMethod(method);
        header.setOriginalSize(payload.length);
        header.setCompressedSize(packed.size());
        header.setCRC((int) crc.getValue());
        String charset = args.length == 7 ? args[6] : "UTF-8";
        byte[] encodedHeader = header.getBytes(charset);
        if (!new LhaHeader(encodedHeader, charset).getPath().equals(header.getPath())) throw new IOException("Independent header path mismatch");
        try (OutputStream output = Files.newOutputStream(Path.of(args[4]))) {
            output.write(encodedHeader);
            packed.writeTo(output);
            output.write(0);
        }
        System.out.println(encodedHeader.length + " " + packed.size() + " " + payload.length + " " + crc.getValue());
    }
}
