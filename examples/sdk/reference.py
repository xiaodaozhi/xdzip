#!/usr/bin/env python3
"""Independent stdlib-only XDP1/CBOR peer for trusted SDK conformance.

This is a test example, not a package accepted by the native sandbox launcher.
No Rust encoder, generated schema or native archive tool is used.
"""
import hashlib
import os
import struct
import sys
import time

MAX_FRAME = 1024 * 1024
MAX_BLOCK = 256 * 1024


def head(major, value):
    for additional, size in ((24, 1), (25, 2), (26, 4), (27, 8)):
        if value < 24:
            return bytes([(major << 5) | value])
        if value < (1 << (size * 8)):
            return bytes([(major << 5) | additional]) + value.to_bytes(size, "big")
    raise ValueError("integer")


def encode(value):
    if isinstance(value, bool):
        return b"\xf5" if value else b"\xf4"
    if isinstance(value, int) and value >= 0:
        return head(0, value)
    if isinstance(value, (bytes, str)):
        raw = value.encode("utf-8") if isinstance(value, str) else value
        return head(3 if isinstance(value, str) else 2, len(raw)) + raw
    if isinstance(value, list):
        return head(4, len(value)) + b"".join(encode(item) for item in value)
    if isinstance(value, dict):
        return head(5, len(value)) + b"".join(
            encode(key) + encode(value[key]) for key in sorted(value)
        )
    raise ValueError("type")


def decode(raw):
    at = 0
    nodes = 0

    def take(size):
        nonlocal at
        if size < 0 or size > len(raw) - at:
            raise ValueError("truncated")
        result = raw[at : at + size]
        at += size
        return result

    def item(depth):
        nonlocal nodes
        nodes += 1
        if depth > 16 or nodes > 8192:
            raise ValueError("limit")
        initial = take(1)[0]
        if initial in (244, 245):
            return initial == 245
        major, extra = initial >> 5, initial & 31
        if major not in (0, 2, 3, 4, 5) or extra > 27:
            raise ValueError("type")
        size = 0 if extra < 24 else (1 << (extra - 24))
        value = extra if size == 0 else int.from_bytes(take(size), "big")
        if size and value < (24 if size == 1 else 1 << (size * 4)):
            raise ValueError("nonminimal")
        if major == 0:
            return value
        if major in (2, 3):
            if value > MAX_BLOCK:
                raise ValueError("block")
            data = take(value)
            return data if major == 2 else data.decode("utf-8", errors="strict")
        if major == 4:
            if value > 128:
                raise ValueError("array")
            return [item(depth + 1) for _ in range(value)]
        if value > 32:
            raise ValueError("map")
        result = {}
        for _ in range(value):
            key = item(depth + 1)
            if type(key) is not int or key in result:
                raise ValueError("key")
            result[key] = item(depth + 1)
        return result

    result = item(0)
    if at != len(raw):
        raise ValueError("trailing")
    return result


def exact(size):
    result = bytearray()
    while len(result) < size:
        data = sys.stdin.buffer.read(size - len(result))
        if not data:
            raise ValueError("eof")
        result.extend(data)
    return bytes(result)


class Peer:
    def __init__(self, fault):
        self.sequence, self.next_sequence = 1, 1
        self.request, self.operation = 0, 0
        self.source, self.length = b"", 0
        self.session = b"python-session!!"
        self.fault = fault

    def send(self, kind, payload):
        wire = encode({1: kind, 2: self.request, 3: self.operation, 4: self.sequence, 5: payload})
        self.sequence += 1
        sys.stdout.buffer.write(struct.pack("<I", len(wire)) + wire)
        sys.stdout.buffer.flush()

    def receive(self, new=False):
        length = struct.unpack("<I", exact(4))[0]
        if not 0 < length <= MAX_FRAME:
            raise ValueError("frame")
        frame = decode(exact(length))
        if not {1, 2, 3, 4, 5}.issubset(frame) or any(key < 128 and key not in {1, 2, 3, 4, 5} for key in frame) or frame[4] != self.next_sequence:
            raise ValueError("envelope")
        self.next_sequence += 1
        if new and frame[1] != 52:
            if frame[2] != self.request + 1 or frame[3] == 0:
                raise ValueError("request")
            if self.operation and frame[3] != self.operation:
                raise ValueError("operation")
            self.request, self.operation = frame[2], frame[3]
        if (frame[2], frame[3]) != (self.request, self.operation):
            raise ValueError("identity")
        if frame[1] == 50:
            self.send(51, {})
            self.send(61, {1: 1})
            raise SystemExit(0)
        return frame[1], frame[5]

    def read(self, offset, size):
        if not 0 < size <= MAX_BLOCK or offset + size > self.length:
            raise ValueError("range")
        token = b"foreign-source!!" if self.fault == "source" else self.source
        actual = (1 << 64) - 1 if self.fault == "overflow" else offset
        self.send(20, {1: token, 2: actual, 3: size})
        kind, payload = self.receive()
        if kind != 21 or payload[1] != self.source or payload[2] != offset or len(payload[3]) != size:
            raise ValueError("read")
        return payload[3]

    def run(self):
        if self.fault == "frame":
            sys.stdout.buffer.write(struct.pack("<I", MAX_FRAME + 1))
            sys.stdout.buffer.flush()
            return
        major = 2 if self.fault == "version" else 1
        hello = {1: major, 2: major, 3: "org.xdzip.sdk.reference", 4: "xdp1"}
        if self.fault == "required":
            hello[6] = 0
        self.send(1, hello)
        if self.receive()[0] != 2:
            raise ValueError("welcome")
        records = []
        while True:
            kind, payload = self.receive(new=True)
            if kind == 10:
                self.source, self.length = payload[1], payload[2]
                if self.fault == "replay":
                    self.sequence -= 1
                if self.fault == "operation":
                    self.operation += 1
                if self.read(0, 4) != b"XDP1":
                    raise ValueError("format")
                self.send(61, {1: 0, 2: "xdp1"})
            elif kind == 11:
                self.send(61, {1: 0, 2: self.session})
            elif kind == 12:
                count = int.from_bytes(self.read(4, 4), "little")
                if count > 1000:
                    raise ValueError("count")
                offset = 8
                for index in range(count):
                    header = self.read(offset, 6)
                    name_size, size = struct.unpack("<HI", header)
                    name = self.read(offset + 6, name_size)
                    offset += 6 + name_size
                    token = struct.pack("<I", 0 if self.fault == "duplicate" else index)
                    path = b"../escape" if self.fault == "path" else name
                    records.append((token, size, offset))
                    self.send(30, {1: self.session, 2: [{1: token, 2: path, 3: 0, 4: size}]})
                    offset += size
                    if offset > self.length:
                        raise ValueError("offset")
                if self.fault != "incomplete":
                    self.send(31, {1: count})
                self.send(61, {1: 0})
            elif kind == 13:
                if self.fault == "hang":
                    time.sleep(30)
                    return
                if self.fault == "pause":
                    time.sleep(0.6)
                token, size, offset = next(r for r in records if r[0] == payload[2])
                sink, credit = payload[3], payload[6]
                digest, written, block = hashlib.sha256(), 0, 0
                while written < size:
                    data = self.read(offset + written, min(size - written, MAX_BLOCK))
                    digest.update(data)
                    target = b"foreign-sink!!!!" if self.fault == "sink" else sink
                    self.send(40, {1: target, 2: block, 3: data, 4: credit})
                    response_kind, ack = self.receive()
                    if response_kind != 42 or ack[1] != sink or ack[2] != block + 1:
                        raise ValueError("credit")
                    if self.fault != "credit":
                        credit = ack[3]
                    block += 1
                    written += len(data)
                    if self.fault == "crash":
                        os._exit(1)
                checksum = bytes(32) if self.fault == "digest" else digest.digest()
                self.send(41, {1: sink, 2: written, 3: checksum})
                self.send(61, {1: 0})
                if self.fault == "terminal":
                    self.send(61, {1: 0})
            elif kind == 14:
                self.send(61, {1: 0})
            elif kind == 52:
                return
            else:
                raise ValueError("state")


if __name__ == "__main__":
    try:
        Peer(sys.argv[1] if len(sys.argv) == 2 else "").run()
    except (ValueError, OSError, KeyError, StopIteration):
        # Peer-controlled text and OS paths never enter host diagnostics.
        raise SystemExit(1) from None
