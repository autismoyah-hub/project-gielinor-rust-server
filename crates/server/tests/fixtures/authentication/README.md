# Public authentication test fixture (issue190)

All three `.hex` files are newly authored **PUBLIC TEST ONLY** data. They do not
come from a recorded login, private reference key, launcher key, cache, user or
machine. They must never be used for security or production authentication.

`rsa-exponent.hex` is the deliberately public RSA PRIVATE exponent `d`, not the
public exponent. Public exponent `e=65537`; factors `p=2^521-1` and `q=2^607-1`
are independently selected mathematical test constants. The bounded Lucas-Lehmer
checks in the recipe prove both factors prime. Their product is1128 bits. There
is no identity-decryption exponent1 shortcut, random key generation, external
library, new dependency or production key reference. The key has no secrecy.

`login.hex` is a raw protocol239 login payload (not the enclosing TCP opcode/
length frame), with a real modular RSA ciphertext and32-round XTEA body. Every
username/password/seed/device/cache field is synthetic. The username is
`public_auth_190`, password `public-fixture-only-190`, OTP absent. Cache values
are dummy checksum integers, not claims about a real cache. Platform bytes are
all zero and the machine ID is bytes0..23, never a collected fingerprint.

The unconditional Rust test
`authentication::tests::public_fixture_login_decodes_without_external_assets`
uses the actual public decoder and asserts the header, credential, username,
XTEA/body fields, ordered CRCs and negative header/RSA/truncation controls. It
uses no DB, Argon2 DLL, environment or input path lookup. Existing interrupted
registration tests still check credential refusal, same CID and checkpoint
resumption using this fixture; their optional native-asset early return remains
qualified and is not proof of an executed authentication scenario.

## Reproduce the exact bytes

The following complete Python3 standard-library recipe returns the three file
texts, each a single ASCII hex line with LF. It reads no input and uses no random
or machine source. Calling `produce()` recomputes factors, key, RSA ciphertext
and XTEA payload; compare each returned text to its named file. For an explicitly
new reproduction directory, write the returned texts using exclusive-create
mode. Do not overwrite test assets while checking reproducibility.

```python
# PUBLIC TEST ONLY: deterministic, deliberately insecure raw RSA/XTEA fixture.
import math
import struct

def mersenne(exponent):
    if exponent not in (521, 607):
        raise ValueError("fixed public test factors only")
    if any(exponent % d == 0 for d in range(2, math.isqrt(exponent) + 1)):
        raise ValueError("nonprime exponent")
    value = (1 << exponent) - 1
    state = 4
    for _ in range(exponent - 2):  # finite Lucas-Lehmer primality check
        state = (state * state - 2) % value
    if state != 0:
        raise ValueError("nonprime Mersenne factor")
    return value

def xtea(data, key, decrypt=False):
    out = bytearray(data)
    mask = 0xffffffff
    for offset in range(0, len(out) - 7, 8):
        a, b = struct.unpack_from(">II", out, offset)
        total = (0x9e3779b9 * 32) & mask if decrypt else 0
        for _ in range(32):
            if decrypt:
                b = (b - ((((a << 4) ^ (a >> 5)) + a) ^ (total + key[(total >> 11) & 3]))) & mask
                total = (total - 0x9e3779b9) & mask
                a = (a - ((((b << 4) ^ (b >> 5)) + b) ^ (total + key[total & 3]))) & mask
            else:
                a = (a + ((((b << 4) ^ (b >> 5)) + b) ^ (total + key[total & 3]))) & mask
                total = (total + 0x9e3779b9) & mask
                b = (b + ((((a << 4) ^ (a >> 5)) + a) ^ (total + key[(total >> 11) & 3]))) & mask
        struct.pack_into(">II", out, offset, a, b)
    return bytes(out)

def produce():
    p, q, e = mersenne(521), mersenne(607), 65537
    n = p * q
    d = pow(e, -1, (p - 1) * (q - 1))
    if d == 1 or n.bit_length() != 1128:
        raise ValueError("nonidentity test RSA required")
    key = (0x10203040, 0x50607080, 0x90a0b0c0, 0xd0e0f001)
    auth = (b"\x01" + struct.pack(">4IQ", *key, 0x0102030405060708)
            + b"\x02" + bytes(4) + b"\x00public-fixture-only-190\x00")
    message = int.from_bytes(auth, "big")
    if not 0 < message < n:
        raise ValueError("RSA plaintext range")
    cipher = pow(message, e, n)
    if cipher == message or pow(cipher, d, n) != message:
        raise ValueError("RSA roundtrip")
    rsa = cipher.to_bytes((n.bit_length() + 7) // 8, "big")
    body = (b"public_auth_190\x00" + b"\x03" + struct.pack(">HH", 765, 503)
            + bytes(range(24)) + b"public-test-only\x00" + struct.pack(">I", 0)
            + b"\x02" + struct.pack(">II", 0x11223344, 0x55667788))
    # HostPlatformStats: fixed bytes, four+two+two empty prefixed strings.
    body += bytes(18) + b"\x00\x00" * 4 + bytes(3) + b"\x00\x00" * 2 + bytes(18) + b"\x00\x00" * 2
    body += b"\xab" + struct.pack(">I", 0x12345678)
    orders = ((18,"M"),(19,"M"),(22,"B"),(12,"L"),(9,"L"),(1,"M"),
              (17,"M"),(16,"I"),(7,"I"),(13,"L"),(3,"B"),(5,"M"),
              (10,"I"),(11,"B"),(2,"M"),(8,"L"),(0,"I"),(21,"B"),
              (4,"I"),(20,"M"),(6,"L"),(14,"I"),(15,"L"))
    for archive, order in orders:
        raw = struct.pack(">I", 0xa5000000 | archive)
        permutation = {"B": (0,1,2,3), "L": (3,2,1,0), "M": (2,3,0,1), "I": (1,0,3,2)}[order]
        body += bytes(raw[i] for i in permutation)
    encrypted = xtea(body, key)
    if xtea(encrypted, key, True) != body or encrypted == body:
        raise ValueError("XTEA roundtrip")
    # Public protocol payload, not outer TCP login opcode/packet-length framing.
    login = struct.pack(">III3BH", 239, 1, 0, 1, 0, 0, len(rsa)) + rsa + encrypted
    integer = lambda v: v.to_bytes((v.bit_length() + 7) // 8, "big").hex() + "\n"
    return {"rsa-exponent.hex": integer(d), "rsa-modulus.hex": integer(n),
            "login.hex": login.hex() + "\n"}
```

Pure recipe/roundtrip checks do not execute the Rust test or prove native storage/
password behavior. Cargo compilation and required named Rust tests need a separate
build grant and independent evidence. No production authentication code, Secret
constructor, protocol implementation or launcher changes belong to this fixture.
