// Owned API harness; reference implementation code remains outside the product.
import java.io.*;
import java.nio.file.*;
import java.util.*;
import jp.gr.java_conf.dangan.util.lha.*;

public final class Reference {
    public static void main(String[] args) throws Exception {
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
        if (args.length != 5 || !args[0].equals("encode")) throw new IllegalArgumentException("Invalid owned harness command");
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
        LhaHeader header = new LhaHeader("hello.bin", new Date(1700000000000L));
        header.setHeaderLevel(level);
        header.setCompressMethod(method);
        header.setOriginalSize(payload.length);
        header.setCompressedSize(packed.size());
        header.setCRC((int) crc.getValue());
        byte[] encodedHeader = header.getBytes("UTF-8");
        try (OutputStream output = Files.newOutputStream(Path.of(args[4]))) {
            output.write(encodedHeader);
            packed.writeTo(output);
            output.write(0);
        }
        System.out.println(encodedHeader.length + " " + packed.size() + " " + payload.length + " " + crc.getValue());
    }
}
