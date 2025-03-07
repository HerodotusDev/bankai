# Used for writing a 384 bit integer to a dict, e.g. a Cairo struct
def write_uint384(ptr, value: int):
    mask = (1 << 96) - 1  # Creates a mask of 96 1's in binary
    for i in range(4):
        chunk = value & mask
        setattr(ptr, f'd{i}', chunk)
        value >>= 96  # Shift right by 96 bits

# Split int into 4 96-bit chunks
def int_to_uint384(value: int) -> list[int]:
    mask = (1 << 96) - 1  # Creates a mask of 96 1's in binary
    chunks = []
    for i in range(4):
        chunk = value & mask
        chunks.append(chunk)
        value >>= 96  # Shift right by 96 bits
    return chunks

# Creates a G1Point from a point dictionary
def write_g1(ptr, point: dict):
    write_uint384(ptr.x, int(point["x"], 16))
    write_uint384(ptr.y, int(point["y"], 16))

# Creates a G2Point from a point dictionary
def write_g2(ptr, point: dict):
    write_uint384(ptr.x0, int(point["x0"], 16))
    write_uint384(ptr.x1, int(point["x1"], 16))
    write_uint384(ptr.y0, int(point["y0"], 16))
    write_uint384(ptr.y1, int(point["y1"], 16))

# Creates a G1G2Pair
def write_g1g2(ptr, g1: dict, g2: dict):
    write_g1(ptr.P, g1)
    write_g2(ptr.Q, g2)

# Convert list of pubkeys to array of uint384
def generate_signers_array(pubs: list[dict]):
    values = []
    for pub in pubs:
        x_chunks = int_to_uint384(int(pub["x"], 16))
        y_chunks = int_to_uint384(int(pub["y"], 16))
        values.append([x_chunks, y_chunks])
    return values

def split_uint256(value: int):
    return [value & 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF, value >> 128]

# This function chunks from MSB to LSB
def hex_to_chunks_32(hex_string: str):
    # Remove '0x' prefix if present
    if hex_string.startswith(('0x', '0X')):
        hex_string = hex_string[2:]

    # if we have an odd number of characters, prepend a 0
    if len(hex_string) % 2 == 1:
        hex_string = '0' + hex_string

    # Now split into 8-character (32-bit) chunks
    chunks = [int(hex_string[i:i+8], 16) for i in range(0, len(hex_string), 8)]
    return chunks


def print_u256(msg, ptr):
    print(f"{msg}: {hex(ptr.high * 2**128 + ptr.low)}")

def print_u384(msg, ptr):
    print(f"{msg}: {hex(ptr.d3 * 2**288 + ptr.d2 * 2**192 + ptr.d1 * 2**96 + ptr.d0)}")

def print_g1(msg, ptr):
    print(f"{msg}:")
    print_u384("x", ptr.x)
    print_u384("y", ptr.y)

def print_g2(msg, ptr):
    print(f"{msg}:")
    print_u384("x0", ptr.x0)
    print_u384("x1", ptr.x1)
    print_u384("y0", ptr.y0)
    print_u384("y1", ptr.y1)

def uint256_to_int(ptr):
    return ptr.high * 2**128 + ptr.low

def int_to_uint256(value: int):
    return [value & 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF, value >> 128]

def uint384_to_int(ptr):
    return ptr.d3 * 2**288 + ptr.d2 * 2**192 + ptr.d1 * 2**96 + ptr.d0

def hex_to_bytes(h: str) -> bytes:
    return bytes.fromhex(h.removeprefix("0x"))
